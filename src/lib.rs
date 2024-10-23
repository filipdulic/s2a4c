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
//! Here is a simple example demonstrating how to use the `Router' struct:
//! ```rust
//! use async_channel::{bounded, Sender, Receiver};
//! use tokio::time::Duration;
//! use s2a4c::router::Router;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Define a timeout
//!     let timeout = Duration::from_millis(100);
//!     // define a worker function
//!     async fn worker(receiver: Receiver<(Uuid, String)>, sender: Sender<(Uuid, String)>) {
//!         while let Ok((uuid, request)) = receiver.recv().await {
//!             sender.send((uuid, "World!".to_string())).await.unwrap();
//!         }
//!     }
//!     // Create a Router
//!     let router: Router<String, String> = Router::default();
//!     // Spawn the workers
//!     router.tokio_spawn_workers(4, worker);
//!     // Spawn the router
//!     router.tokio_spawn();
//!
//!     // Make a request to the endpoint
//!     let request = "Hello".to_string();
//!     let response = router
//!         .endpoint(Some(timeout))
//!         .handle_request(request.clone())
//!         .await;
//!     assert!(response.is_ok());
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
    use crate::{endpoint::EndpointError, router::Router};
    use async_channel::{Receiver, Sender};
    use test_case::test_case;
    use tokio::time::Duration;
    use uuid::Uuid;

    #[test_case(250, true)]
    #[test_case(150, false)]
    #[tokio::test]
    async fn test_endpoint_and_router_with_timeout(timeout_in_msecs: u64, is_ok: bool) {
        // Define a timeout and response delay
        let timeout = Duration::from_millis(timeout_in_msecs);
        // Create a Router
        let router: Router<String, String> = Router::default();

        async fn worker_200ms(receiver: Receiver<(Uuid, String)>, sender: Sender<(Uuid, String)>) {
            while let Ok((uuid, request)) = receiver.recv().await {
                tokio::time::sleep(Duration::from_millis(200)).await;
                let response = format!("Response to request: {}", request);
                sender.send((uuid, response)).await.unwrap();
            }
        }
        // Spawn the router
        router.tokio_spawn();
        // Spaen the workers
        router.tokio_spawn_workers(4, worker_200ms);

        // Make a request to the endpoint
        let request = "Hello, world!".to_string();
        let response = router
            .endpoint(Some(timeout))
            .handle_request(request.clone())
            .await;
        assert_eq!(response.is_ok(), is_ok);
        match response {
            Ok(response) => {
                assert_eq!(response, format!("Response to request: {}", request));
            }
            Err(err) => {
                assert!(matches!(err, EndpointError::Timeout(_)));
            }
        }
    }
}
