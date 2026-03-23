use super::ast::*;
use lifelog_core::DataOrigin;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum ExecutionPlan {
    /// A query targeted at a specific table/origin.
    TableQuery {
        origin: DataOrigin,
        filter: Option<Expression>,
        limit: usize,
    },
    /// Multiple queries to be executed.
    MultiQuery(Vec<ExecutionPlan>),
    /// Two-stage temporal join for `DURING(...)` / `OVERLAPS(...)` / `WITHIN(...)`.
    ///
    /// Phase 1: query the `source_*` tables for candidate intervals.
    /// Phase 2: query the target table for UUIDs whose timestamps fall within any interval,
    /// in addition to the target base predicate.
    DuringQuery {
        target_origin: DataOrigin,
        target_base_filter: Option<Expression>,
        during_terms: Vec<DuringTermPlan>,
        target_limit: usize,
    },
    /// Placeholder for multi-stage plans.
    #[allow(dead_code)]
    Unsupported(String),
}

#[derive(Debug, PartialEq)]
pub struct DuringSourcePlan {
    pub source_origin: DataOrigin,
    pub filter: Expression,
}

#[derive(Debug, PartialEq)]
pub struct DuringTermPlan {
    pub source_plans: Vec<DuringSourcePlan>,
    pub window: chrono::Duration,
}

pub struct Planner;

impl Planner {
    pub fn plan(query: &Query, available_origins: &[DataOrigin]) -> ExecutionPlan {
        let origins = Self::resolve_selector(&query.target, available_origins);

        if origins.is_empty() {
            return ExecutionPlan::MultiQuery(vec![]);
        }

        const DEFAULT_MAX_TARGET_UUIDS: usize = 1_000;

        let plan_ctx = PlanContext {
            available_origins,
            max_target_uuids: DEFAULT_MAX_TARGET_UUIDS,
        };

        let plans = origins
            .into_iter()
            .map(|origin| {
                let origin_ctx = OriginPlanContext {
                    plan: &plan_ctx,
                    origin: &origin,
                };
                // If the query has temporal operators under boolean ORs, plan by converting the
                // predicate to a bounded DNF (OR-of-ANDs) and unioning the resulting conjunctive plans.
                //
                // This allows queries like: `(DURING(...) OR DURING(...)) AND leaf_predicate`.
                //
                // NOTE: NOT over temporal operators remains unsupported (would require set difference).
                if Self::contains_temporal_ops(&query.filter) && Self::contains_or(&query.filter) {
                    const MAX_DNF_CONJUNCTIONS: usize = 16;
                    let conjs = match Self::to_bounded_dnf_conjunctions(
                        &query.filter,
                        MAX_DNF_CONJUNCTIONS,
                    ) {
                        Ok(v) => v,
                        Err(msg) => return ExecutionPlan::Unsupported(msg),
                    };

                    let mut subplans = Vec::new();
                    for conj in conjs {
                        match Self::compile_conjunctive(&conj) {
                            Ok((sql_terms, temporal_terms)) => {
                                subplans.push(Self::plan_conjunctive_for_origin(
                                    &origin_ctx,
                                    sql_terms,
                                    temporal_terms,
                                ))
                            }
                            Err(msg) => return ExecutionPlan::Unsupported(msg),
                        }
                    }

                    return Self::multi(subplans);
                }

                match Self::compile_conjunctive(&query.filter) {
                    Ok((sql_terms, temporal_terms)) => {
                        Self::plan_conjunctive_for_origin(&origin_ctx, sql_terms, temporal_terms)
                    }
                    Err(msg) => ExecutionPlan::Unsupported(msg),
                }
            })
            .collect();

        Self::multi(plans)
    }

    fn plan_conjunctive_for_origin(
        ctx: &OriginPlanContext<'_>,
        filter_terms: Vec<Expression>,
        temporal_terms: Vec<TemporalTerm>,
    ) -> ExecutionPlan {
        let target_base_filter = Self::combine_with_and(filter_terms);

        if temporal_terms.is_empty() {
            return ExecutionPlan::TableQuery {
                origin: ctx.origin.clone(),
                filter: target_base_filter,
                limit: ctx.plan.max_target_uuids,
            };
        }

        let mut during_terms = Vec::new();
        for t in temporal_terms {
            let term = match t {
                TemporalTerm::Within(d) | TemporalTerm::During(d) | TemporalTerm::Overlaps(d) => d,
            };

            let source_origins = Self::resolve_selector(&term.stream, ctx.plan.available_origins);
            if source_origins.is_empty() {
                return ExecutionPlan::TableQuery {
                    origin: ctx.origin.clone(),
                    filter: None,
                    limit: 0,
                };
            }

            if Self::contains_temporal_ops(&term.predicate) {
                return ExecutionPlan::Unsupported(
                    "Nested temporal operators inside temporal predicates are not supported yet"
                        .to_string(),
                );
            }

            let source_plans = source_origins
                .into_iter()
                .map(|source_origin| DuringSourcePlan {
                    source_origin,
                    filter: term.predicate.clone(),
                })
                .collect();

            during_terms.push(DuringTermPlan {
                source_plans,
                window: term.window,
            });
        }

        ExecutionPlan::DuringQuery {
            target_origin: ctx.origin.clone(),
            target_base_filter,
            during_terms,
            target_limit: ctx.plan.max_target_uuids,
        }
    }

    /// Attempts to decompose `expr` into a conjunction of SQL-compilable terms plus
    /// one or more top-level temporal join terms (`WITHIN(...)` and/or `DURING(...)`).
    ///
    /// Current limitation (intentional): `WITHIN` cannot appear under `OR` / `NOT`.
    fn compile_conjunctive(
        expr: &Expression,
    ) -> Result<(Vec<Expression>, Vec<TemporalTerm>), String> {
        let mut filter_terms: Vec<Expression> = Vec::new();
        let mut temporal_terms: Vec<TemporalTerm> = Vec::new();

        // Flatten nested ANDs iteratively to keep stack shallow.
        let mut queue: VecDeque<&Expression> = VecDeque::new();
        queue.push_back(expr);

        while let Some(node) = queue.pop_front() {
            match node {
                Expression::And(l, r) => {
                    queue.push_back(l);
                    queue.push_back(r);
                }
                Expression::Within {
                    stream,
                    predicate,
                    window,
                } => temporal_terms.push(TemporalTerm::Within(DuringTerm {
                    stream: stream.clone(),
                    predicate: (**predicate).clone(),
                    window: *window,
                })),
                Expression::During {
                    stream,
                    predicate,
                    window,
                } => {
                    temporal_terms.push(TemporalTerm::During(DuringTerm {
                        stream: stream.clone(),
                        predicate: (**predicate).clone(),
                        window: *window,
                    }));
                }
                Expression::Overlaps {
                    stream,
                    predicate,
                    window,
                } => {
                    temporal_terms.push(TemporalTerm::Overlaps(DuringTerm {
                        stream: stream.clone(),
                        predicate: (**predicate).clone(),
                        window: *window,
                    }));
                }
                Expression::Or(..) | Expression::Not(..) => {
                    if Self::contains_temporal_ops(node) {
                        return Err(
                            "Temporal joins (WITHIN/DURING/OVERLAPS) are only supported under conjunctions (AND), not OR/NOT"
                                .to_string(),
                        );
                    }
                    filter_terms.push(node.clone());
                }
                _ => filter_terms.push(node.clone()),
            }
        }

        Ok((filter_terms, temporal_terms))
    }

    fn multi(plans: Vec<ExecutionPlan>) -> ExecutionPlan {
        let mut flat = Vec::new();
        for p in plans {
            match p {
                ExecutionPlan::MultiQuery(inner) => flat.extend(inner),
                other => flat.push(other),
            }
        }
        ExecutionPlan::MultiQuery(flat)
    }

    fn contains_temporal_ops(expr: &Expression) -> bool {
        match expr {
            Expression::Within { .. } | Expression::During { .. } | Expression::Overlaps { .. } => {
                true
            }
            Expression::And(l, r) | Expression::Or(l, r) => {
                Self::contains_temporal_ops(l) || Self::contains_temporal_ops(r)
            }
            Expression::Not(inner) => Self::contains_temporal_ops(inner),
            _ => false,
        }
    }

    fn contains_or(expr: &Expression) -> bool {
        match expr {
            Expression::Or(..) => true,
            Expression::And(l, r) => Self::contains_or(l) || Self::contains_or(r),
            Expression::Not(inner) => Self::contains_or(inner),
            Expression::Within { predicate, .. }
            | Expression::During { predicate, .. }
            | Expression::Overlaps { predicate, .. } => Self::contains_or(predicate),
            _ => false,
        }
    }

    /// Convert an expression into a bounded disjunctive-normal-form (DNF) representation, returning a
    /// set of conjunction expressions whose union is equivalent to the input.
    ///
    /// This is intentionally minimal: it only distributes AND over OR. NOT is treated as an atom.
    fn to_bounded_dnf_conjunctions(
        expr: &Expression,
        max_conjunctions: usize,
    ) -> Result<Vec<Expression>, String> {
        fn dnf(expr: &Expression, max: usize) -> Result<Vec<Vec<Expression>>, String> {
            match expr {
                Expression::And(l, r) => {
                    let left = dnf(l, max)?;
                    let right = dnf(r, max)?;
                    let mut out = Vec::new();
                    for lc in left {
                        for rc in &right {
                            let mut conj = lc.clone();
                            conj.extend(rc.clone());
                            out.push(conj);
                            if out.len() > max {
                                return Err(format!(
                                    "Query boolean expansion too large (DNF conjunctions > {max}); simplify the query"
                                ));
                            }
                        }
                    }
                    Ok(out)
                }
                Expression::Or(l, r) => {
                    let mut out = dnf(l, max)?;
                    let right = dnf(r, max)?;
                    out.extend(right);
                    if out.len() > max {
                        return Err(format!(
                            "Query boolean expansion too large (DNF conjunctions > {max}); simplify the query"
                        ));
                    }
                    Ok(out)
                }
                _ => Ok(vec![vec![expr.clone()]]),
            }
        }

        fn atoms_to_expr(atoms: Vec<Expression>) -> Expression {
            if atoms.is_empty() {
                // Equivalent to a tautology. This keeps planning simple without adding a new AST variant.
                return Expression::TimeRange(
                    chrono::DateTime::<chrono::Utc>::MIN_UTC,
                    chrono::DateTime::<chrono::Utc>::MAX_UTC,
                );
            }

            let mut iter = atoms.into_iter();
            let Some(mut acc) = iter.next() else {
                return Expression::TimeRange(
                    chrono::DateTime::<chrono::Utc>::MIN_UTC,
                    chrono::DateTime::<chrono::Utc>::MAX_UTC,
                );
            };
            for a in iter {
                acc = Expression::And(Box::new(acc), Box::new(a));
            }
            acc
        }

        let conjs = dnf(expr, max_conjunctions)?;
        Ok(conjs.into_iter().map(atoms_to_expr).collect())
    }

    fn resolve_selector(
        selector: &StreamSelector,
        available_origins: &[DataOrigin],
    ) -> Vec<DataOrigin> {
        match selector {
            StreamSelector::All => available_origins.to_vec(),
            StreamSelector::Modality(m) => available_origins
                .iter()
                .filter(|o| o.modality_name == *m)
                .cloned()
                .collect(),
            StreamSelector::StreamId(id) => {
                // Try to match exactly by table name first.
                if let Some(origin) = available_origins.iter().find(|o| o.get_table_name() == *id) {
                    vec![origin.clone()]
                } else {
                    // Fallback: search for origins that match this as a suffix or exact modality.
                    let matches: Vec<_> = available_origins
                        .iter()
                        .filter(|o| {
                            o.get_table_name() == *id
                                || o.modality_name == *id
                                || o.get_table_name().ends_with(&format!(":{}", id))
                        })
                        .cloned()
                        .collect();

                    if matches.is_empty() {
                        // Final fallback: try to parse it.
                        if let Ok(origin) = lifelog_core::DataOrigin::tryfrom_string(id.clone()) {
                            vec![origin]
                        } else {
                            vec![]
                        }
                    } else {
                        matches
                    }
                }
            }
        }
    }

    fn combine_with_and(terms: Vec<Expression>) -> Option<Expression> {
        let mut iter = terms.into_iter();
        let mut acc = iter.next()?;
        for term in iter {
            acc = Expression::And(Box::new(acc), Box::new(term));
        }
        Some(acc)
    }
}

struct PlanContext<'a> {
    available_origins: &'a [DataOrigin],
    max_target_uuids: usize,
}

struct OriginPlanContext<'a> {
    plan: &'a PlanContext<'a>,
    origin: &'a DataOrigin,
}

#[derive(Debug, Clone, PartialEq)]
struct DuringTerm {
    stream: StreamSelector,
    predicate: Expression,
    window: chrono::Duration,
}

#[derive(Debug, Clone, PartialEq)]
enum TemporalTerm {
    Within(DuringTerm),
    During(DuringTerm),
    Overlaps(DuringTerm),
}

#[cfg(test)]
mod tests {
    use super::*;
    use lifelog_core::{DataOrigin, DataOriginType};

    #[test]
    #[allow(clippy::panic)]
    fn test_planner_origin_resolution() {
        let origins = vec![
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Screen".into()),
            DataOrigin::new(DataOriginType::DeviceId("phone".into()), "Browser".into()),
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Browser".into()),
        ];

        // 1. Exact match by table name
        let query = Query {
            target: StreamSelector::StreamId("laptop:Screen".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 1);
            if let ExecutionPlan::TableQuery { origin, .. } = &plans[0] {
                assert_eq!(origin.get_table_name(), "laptop:Screen");
            } else {
                panic!("Expected TableQuery");
            }
        } else {
            panic!("Expected MultiQuery");
        }

        // 2. Match by suffix (e.g. just "Browser" -> matches both browsers)
        let query = Query {
            target: StreamSelector::StreamId("Browser".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 2);
            let tables: Vec<_> = plans
                .iter()
                .map(|p| match p {
                    ExecutionPlan::TableQuery { origin, .. } => origin.get_table_name(),
                    _ => "".to_string(),
                })
                .collect();
            assert!(tables.contains(&"laptop:Browser".to_string()));
            assert!(tables.contains(&"phone:Browser".to_string()));
        } else {
            panic!("Expected MultiQuery");
        }

        // 3. Fallback to direct parsing if no match found
        let query = Query {
            target: StreamSelector::StreamId("unknown:device".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 1);
            if let ExecutionPlan::TableQuery { origin, .. } = &plans[0] {
                assert_eq!(origin.get_table_name(), "unknown:device");
            } else {
                panic!("Expected TableQuery");
            }
        }
    }

    #[test]
    #[allow(clippy::panic)]
    fn plans_temporal_or_via_dnf_union() {
        let origins = vec![
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Audio".into()),
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Browser".into()),
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Ocr".into()),
        ];

        let query = Query {
            target: StreamSelector::Modality("Audio".into()),
            filter: Expression::Or(
                Box::new(Expression::During {
                    stream: StreamSelector::Modality("Browser".into()),
                    predicate: Box::new(Expression::Contains("url".into(), "youtube".into())),
                    window: chrono::Duration::seconds(30),
                }),
                Box::new(Expression::During {
                    stream: StreamSelector::Modality("Ocr".into()),
                    predicate: Box::new(Expression::Contains("text".into(), "3Blue1Brown".into())),
                    window: chrono::Duration::seconds(30),
                }),
            ),
        };

        let plan = Planner::plan(&query, &origins);
        let ExecutionPlan::MultiQuery(plans) = plan else {
            panic!("expected MultiQuery");
        };

        // OR should become a union of 2 conjunctive plans for the single target origin.
        assert_eq!(plans.len(), 2);
        assert!(plans
            .iter()
            .all(|p| matches!(p, ExecutionPlan::DuringQuery { .. })));
    }
}
