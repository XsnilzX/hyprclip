/// Convenient alias for results used across the crate.
pub type AnyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
