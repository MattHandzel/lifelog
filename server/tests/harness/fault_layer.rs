//! Tower middleware for fault injection in integration tests.
//!
//! `FaultController` holds a shared list of `FaultRule`s that the `FaultInjectionLayer`
//! evaluates on every gRPC request. Tests add/remove rules via the controller; the
//! layer intercepts requests and applies matching faults (reject, delay, drop).

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::RwLock;
use tonic::Status;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// Fault rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum FaultRule {
    /// Reject the next N requests with the given gRPC status code.
    RejectNextN {
        remaining: Arc<AtomicU64>,
        code: tonic::Code,
        message: String,
    },
    /// Add a fixed delay to every request.
    #[allow(dead_code)]
    DelayAll { duration: std::time::Duration },
    /// Drop (reject) every Nth request. Counter is shared.
    DropEveryN { n: u64, counter: Arc<AtomicU64> },
}

impl FaultRule {
    pub fn reject_next_n(n: u64, code: tonic::Code, message: impl Into<String>) -> Self {
        Self::RejectNextN {
            remaining: Arc::new(AtomicU64::new(n)),
            code,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn delay_all(duration: std::time::Duration) -> Self {
        Self::DelayAll { duration }
    }

    pub fn drop_every_n(n: u64) -> Self {
        Self::DropEveryN {
            n,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }
}

// ---------------------------------------------------------------------------
// Controller (shared between test code and the layer)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct FaultController {
    rules: Arc<RwLock<Vec<FaultRule>>>,
}

enum FaultAction {
    Reject(tonic::Code, String),
    Delay(std::time::Duration),
}

impl FaultController {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add_rule(&self, rule: FaultRule) {
        self.rules.write().await.push(rule);
    }

    pub async fn clear(&self) {
        self.rules.write().await.clear();
    }

    /// Evaluate all active rules and return the first fault to apply, if any.
    async fn evaluate(&self) -> Option<FaultAction> {
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            match rule {
                FaultRule::RejectNextN {
                    remaining,
                    code,
                    message,
                } => {
                    let prev = remaining.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                        if v > 0 {
                            Some(v - 1)
                        } else {
                            None
                        }
                    });
                    if prev.is_ok() {
                        return Some(FaultAction::Reject(*code, message.clone()));
                    }
                }
                FaultRule::DelayAll { duration } => {
                    return Some(FaultAction::Delay(*duration));
                }
                FaultRule::DropEveryN { n, counter } => {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if count % n == 0 {
                        return Some(FaultAction::Reject(
                            tonic::Code::Unavailable,
                            "fault: drop_every_n".into(),
                        ));
                    }
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Tower Layer + Service
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FaultInjectionLayer {
    controller: FaultController,
}

impl FaultInjectionLayer {
    pub fn new(controller: FaultController) -> Self {
        Self { controller }
    }
}

impl<S> Layer<S> for FaultInjectionLayer {
    type Service = FaultInjectionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FaultInjectionService {
            inner,
            controller: self.controller.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaultInjectionService<S> {
    inner: S,
    controller: FaultController,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for FaultInjectionService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let controller = self.controller.clone();
        // Clone inner and swap so we have an owned, ready service.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            match controller.evaluate().await {
                Some(FaultAction::Reject(code, msg)) => {
                    // Build an HTTP response with the gRPC error status in headers.
                    let status = Status::new(code, msg);
                    let mut response = http::Response::new(ResBody::default());
                    // Set the gRPC status code in trailers via headers
                    *response.status_mut() = http::StatusCode::OK;
                    response.headers_mut().insert(
                        "grpc-status",
                        http::HeaderValue::from_str(&(status.code() as i32).to_string())
                            .unwrap_or_else(|_| http::HeaderValue::from_static("13")),
                    );
                    response.headers_mut().insert(
                        "grpc-message",
                        http::HeaderValue::from_str(status.message())
                            .unwrap_or_else(|_| http::HeaderValue::from_static("fault injection")),
                    );
                    Ok(response)
                }
                Some(FaultAction::Delay(dur)) => {
                    tokio::time::sleep(dur).await;
                    inner.call(req).await
                }
                None => inner.call(req).await,
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_controller_reject_next_n() {
        let ctrl = FaultController::new();
        ctrl.add_rule(FaultRule::reject_next_n(
            2,
            tonic::Code::Unavailable,
            "test",
        ))
        .await;

        assert!(ctrl.evaluate().await.is_some());
        assert!(ctrl.evaluate().await.is_some());
        assert!(ctrl.evaluate().await.is_none());
    }

    #[tokio::test]
    async fn test_controller_clear() {
        let ctrl = FaultController::new();
        ctrl.add_rule(FaultRule::reject_next_n(100, tonic::Code::Internal, "test"))
            .await;
        assert!(ctrl.evaluate().await.is_some());

        ctrl.clear().await;
        assert!(ctrl.evaluate().await.is_none());
    }

    #[tokio::test]
    async fn test_drop_every_n() {
        let ctrl = FaultController::new();
        ctrl.add_rule(FaultRule::drop_every_n(3)).await;

        assert!(ctrl.evaluate().await.is_none()); // 1
        assert!(ctrl.evaluate().await.is_none()); // 2
        assert!(ctrl.evaluate().await.is_some()); // 3
        assert!(ctrl.evaluate().await.is_none()); // 4
        assert!(ctrl.evaluate().await.is_none()); // 5
        assert!(ctrl.evaluate().await.is_some()); // 6
    }
}
