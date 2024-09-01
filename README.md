This is a minimal implementation of modal in Rust inspired by Charles Frye's excellent minimodal walkthrough: https://github.com/charlesfrye/minimodal

## Motivation

I think Modal is amazing, and Python is a perfect first use case. 
However, at some point, other languages will likely need to be supported, and I think Rust is the best candidate to be that second language.

1. Rust is statically typed, which means you avoid a whole set of runtime errors that make developing with modal annoying.
    Generally, the thing that annoys me most when using modal is 
    getting dumb runtime errors that could easily have been avoided with a compiler.

2. Rust is becoming more popular in data engineering, and I think it is sufficiently different from Python that it is worth exploring what a Rust-first-class modal might look like. 

3. Serverless orchestration of streaming data pipelines becomes, in my opinion, a lot more interesting in Rust.

This repo is an exploration of what Modal might look like in Rust. 

## Downsides
1. Your environment on the server and client side has to be basically identical. IE you would not be able to do 
    ```rust
    with image.imports():
        import package # not locally installed
    ```
2. Compilation errors might be hard to deal with if you have divergent versions of the code on the server and client side.

## Description
As Rust is far less dynamic than Python, the implementation differs somewhat.

Instead of using a decorator to wrap the function, we use a macro that transforms our function into a struct of the same name
which implements the "Function", "BatchFunction", and "StreamingFunction" traits. These traits are defined as follows:

```rust
pub trait Function<I, O>
where
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static,
    O: Serialize + for<'de> Deserialize<'de> + Send + Sync + Debug + 'static,
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
    // Keeps the futures in the vector
    fn map_async(inputs: Vec<I>) -> Vec<Self::RemoteOutput>;

    // Returns a future that resolves to a vector of results
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
```

Both the inputs and outputs must implement Serialize and Deserialize. This ensures that the arguments can be sent and received via gRPC.
In Python, you can often end up getting pickle errors at runtime when the object you are sending or receiving cannot be deserialized or serialized. Here, the contract is more explicit.

We would then use the macro like so:

```rust
#[function]
async fn add(a: i32, b: i32) -> Result<i32, MiniModalError> {
    a + b
}

#[tokio::main]
async fn main() {
    let result = add::remote((1, 2)).await;
}
```

## Main crates
1. **tonic**: A gRPC framework for Rust, used to implement the client-server communication based on Protocol Buffers.
2. **serde**: Provides serialization and deserialization for Rust data structures, ensuring efficient data transfer between client and server.
3. **tokio**: An asynchronous runtime for Rust, powering the concurrent execution of tasks and handling of I/O operations.
4. **prost**: Works alongside tonic to compile Protocol Buffer definitions into Rust code.
