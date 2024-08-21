//use std::sync::Arc;
use std::any::Any;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use std::future::Future;

pub trait Function<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
{
    type LocalOutput: Future<Output = O> + Send;
    type RemoteOutput: Future<Output = O> + Send;

    fn local(input: I) -> Self::LocalOutput;
    fn remote(input: I) -> Self::RemoteOutput;
}


//TODO implement LocalResult that can take a future and return the value
pub trait LocalResult<T: Send + Sync>: Send {
    fn into_future(self) -> Box<dyn Future<Output = T> + Send + Sync>;
}

impl<T: Send + Sync + 'static> LocalResult<T> for T {
    fn into_future(self) -> Box<dyn Future<Output = T> + Send + Sync> {
        Box::new(std::future::ready(self))
    }
}