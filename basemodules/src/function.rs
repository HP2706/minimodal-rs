use std::sync::Arc;
use erased_serde::{Serializer as ErasedSerializer, Serialize as ErasedSerialize, Deserializer as ErasedDeserialize};
use std::any::Any;

// Define a trait for the input and output types
pub trait FunctionArg: for<'de> ErasedDeserialize<'de> + ErasedSerializer + ErasedSerialize + Send + Sync + Any + 'static {}

impl<T> FunctionArg for T where T: 
ErasedSerializer + for<'de> ErasedDeserialize<'de> + ErasedSerialize + Send + Sync + Any + 'static {}

pub struct Function {
    pub name: String,
    pub local: Arc<dyn Fn(Box<dyn FunctionArg>) -> Box<dyn FunctionArg>>
}

impl Function {
    pub fn new<I, O, F>(name: String, local: F) -> Self
    where
        I: FunctionArg,
        O: FunctionArg,
        F: Fn(I) -> O + Send + Sync + 'static,
    {
        Function {
            name,
            local: Arc::new(move |input: Box<dyn FunctionArg>| {
                // Downcast using Any
                let input = input.as_any().downcast::<I>()
                .expect("Failed to downcast input to the expected type");
                Box::new(local(*input))
            }),
        }
    }
}

// Macro to create a Function
#[macro_export]
macro_rules! create_function {
    ($name:expr, $input:ty, $output:ty, $func:expr) => {
        Function::new::<$input, $output, _>($name.to_string(), $func)
    };
}

// Add this trait to allow downcasting
pub trait AsAny {
    fn as_any(self) -> Box<dyn Any>;
}

impl<T: 'static> AsAny for T {
    fn as_any(self) -> Box<dyn Any> { Box::new(self) }
}