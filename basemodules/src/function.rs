use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use std::future::Future;
use std::iter::IntoIterator;
use crate::MiniModalError;
use futures::Stream;
use std::pin::Pin;

// New trait to encapsulate common requirements
pub trait BaseBound: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static {}

// Implement CommonBounds for all types that meet the requirements
impl<T> BaseBound for T
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static
{}

pub trait Function<I, O>
where
    I: BaseBound,
    O: BaseBound,
{
    type LocalOutput: Future<Output = O> + Send;
    type RemoteOutput: Future<Output = O> + Send;
    
    fn local(input: I) -> Self::LocalOutput;
    fn remote(input: I) -> Self::RemoteOutput;
}

pub trait BatchFunction<I, O>: Function<I, O>
where
    I: BaseBound,
    O: BaseBound,
{
    // keeps the futures in the vector
    fn map_async(inputs: Vec<I>) -> Vec<Self::RemoteOutput>;

    // returns a future that resolves to a vector of results
    fn map(inputs: Vec<I>) -> Pin<Box<dyn Future<Output = Vec<O>> + Send>>;

}

pub trait StreamingFunction<I, O>: Function<I, O>
where
    I: BaseBound,
    O: BaseBound,
{
    type InputStream: Stream<Item = I> + Send;
    type OutputStream: Stream<Item = Self::RemoteOutput> + Send;

    fn map_stream(input: Self::InputStream) -> Self::OutputStream;
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