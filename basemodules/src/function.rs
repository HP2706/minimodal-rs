//use std::sync::Arc;
use std::any::Any;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use std::future::Future;

// Define a trait for the input and output types
//#[typetag::serde(tag = "type")]
pub trait Function<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
{
    type Output: Future<Output = O> + Send;

    fn local(input: I) -> Self::Output;
}