//use std::sync::Arc;
use std::any::Any;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use std::future::Future;
use anyhow::Result;


pub trait Function<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + Any + Debug + 'static,
{
    type LocalOutput: Future<Output = O> + Send;
    //type RemoteOutput: Future<Output = Result<O, anyhow::Error>> + Send;

    fn local(input: I) -> Self::LocalOutput;
    //fn remote(input: I) -> Self::RemoteOutput;
}