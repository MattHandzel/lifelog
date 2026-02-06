use crate::error::LifelogError;

/// Trait for validating proto-generated config types.
///
/// Types that implement `Validate` can check their invariants
/// and return a `LifelogError::Validation` on failure.
pub trait Validate {
    /// Check all invariants. Returns `Ok(())` if valid.
    fn validate(&self) -> Result<(), LifelogError>;
}

// Implementations will be added in lifelog-proto or where the types are available
// to avoid circular dependencies if we want to validate proto types.
// But wait, if lifelog-core doesn't depend on lifelog-proto, it can't implement it here.
// I will put the trait here and implementations in lifelog-proto.
