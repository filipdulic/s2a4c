//! # s2a4c Library
//!
//! Short for `sync to async for comuntication` provides modules for converting
//! synchronious endpoints into asynchronious messagging systems with timeout.
//!
//! ## Modules
//!
//! - [endpoint]: Provides the
//!     [Endpoint](endpoint::Endpoint) struct and
//!     [EndpointError](endpoint::EndpointError) enum for handling
//!     asynchronous communication with a timeout mechanism.
//! - [router]: Provides the [Router](router::Router)
//!     struct for routing request-response communication using
//!     [async-channel](https://docs.rs/async-channel).
//!
//! ## Overview
//!
//! The `s2a4c` library is designed to facilitate asynchronous
//! communication in Rust applications. It leverages the `async_channel` crate
//! for message passing and the `tokio` crate for handling asynchronous
//! operations and timeouts. The library provides a robust and efficient way to
//! manage request-response patterns with unique identifiers and optional
//! timeouts.
//!
//! ## Usage
//!
//! To use the `s2a4c` library, you need to include it as a dependency
//! in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! s2a4c = "0.1.0"
//! ```
//!
//! You can then import the necessary modules and structs in your code:
//!
//! ```rust
//! use s2a4c::endpoint::{Endpoint, EndpointError};
//! use s2a4c::router::Router;
//! ```
//!
//! ## Example
//!
//! Here is a simple example demonstrating how to use the `Endpoint` and
//! `Router` structs:
//!
//! ```rust
//! use async_channel::{bounded, Sender};
//! use tokio::time::Duration;
//! use s2a4c::endpoint::{Endpoint, EndpointError};
//! use s2a4c::router::Router;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), EndpointError> {
//!     // Create an endpoint with a timeout
//!     let (sender, receiver) = bounded(100);
//!     let endpoint = Endpoint::<String, String>::new(sender, Some(Duration::from_secs(5)));
//!
//!     // Create a router
//!     let router: Router<String, String> = Router::default();
//!
//!     // Example usage of the endpoint and router
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - Asynchronous request-response handling
//! - Optional timeout for requests
//!
//! ## Dependencies
//!
//! - [`async-channel`](https://docs.rs/async-channel) for asynchronous message passing
//! - [`tokio`](https://docs.rs/tokio) for asynchronous operations
//! - [`uuid`](https://docs.rs/uuid) for generating unique identifiers
//! - [`scc`](https://docs.rs/scc) for a concurrent HashMap used for mapping UUIDs to respon
//! - [`thiserror`](https://docs.rs/thiserror) for error handling

pub mod endpoint;
pub mod router;

#[cfg(test)]
mod tests {
    use crate::router::Router;
    use async_channel::{Receiver, Sender};
    use tokio::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_endpoint_and_router_with_timeout() {
        // Create a Router
        let router: Router<String, String> = Router::default();

        // Define worker function
        async fn worker_200ms(receiver: Receiver<(Uuid, String)>, sender: Sender<(Uuid, String)>) {
            while let Ok((uuid, request)) = receiver.recv().await {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let response = format!("Response to request: {}", request);
                sender.send((uuid, response)).await.unwrap();
            }
        }
        // Spawn a worker to handle requests
        router.tokio_spawn_workers(4, worker_200ms);
        // Spawn the router loops
        router.tokio_spawn();
    }
}
