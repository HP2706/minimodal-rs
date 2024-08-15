use anyhow::anyhow;
use std::sync::Arc;

pub struct Function<T, U> {
    pub name: String,
    local: Arc<dyn Fn(T) -> U + Send + Sync>,
}

impl<T, U> Function<T, U>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
    U: serde::de::DeserializeOwned + serde::Serialize,
{
    pub fn new(name: String, local: Arc<dyn Fn(T) -> U + Send + Sync>) -> Self {
        Self { name, local }
    }

    pub fn local(&self, args: T) -> U {
        (self.local)(args)
    }
}