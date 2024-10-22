//! # Sync2Async4Coms Library
//!
//! This library provides utilities for managing asynchronous communication in Rust applications. It includes
//! modules for handling endpoints and routing requests and responses using unique identifiers (UUIDs).
//!
//! ## Modules
//!
//! - `endpoint`: Provides the `Endpoint` struct and `EndpointError` enum for handling asynchronous communication
//!   with a timeout mechanism.
//! - `router`: Provides the `Router` struct for managing asynchronous request-response communication using UUIDs.
//!
//! ## Overview
//!
//! The `sync2async4coms` library is designed to facilitate asynchronous communication in Rust applications. It
//! leverages the `async_channel` crate for message passing and the `tokio` crate for handling asynchronous
//! operations and timeouts. The library provides a robust and efficient way to manage request-response patterns
//! with unique identifiers and optional timeouts.
//!
//! ## Usage
//!
//! To use the `sync2async4coms` library, you need to include it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! sync2async4coms = "0.1.0"
//! ```
//!
//! You can then import the necessary modules and structs in your code:
//!
//! ```rust
//! use sync2async4coms::endpoint::{Endpoint, EndpointError};
//! use sync2async4coms::router::Router;
//! ```
//!
//! ## Example
//!
//! Here is a simple example demonstrating how to use the `Endpoint` and `Router` structs:
//!
//! ```rust
//! use async_channel::{bounded, Sender};
//! use tokio::time::Duration;
//! use sync2async4coms::endpoint::{Endpoint, EndpointError};
//! use sync2async4coms::router::Router;
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
//! - Unique identifier (UUID) based routing
//! - Optional timeout for requests
//! - Comprehensive error handling
//!
//! ## Dependencies
//!
//! - `async_channel` for asynchronous message passing
//! - `tokio` for handling asynchronous operations and timeouts
//! - `uuid` for generating unique identifiers
//! - `thiserror` for error handling

pub mod endpoint;
pub mod router;

#[cfg(test)]
mod tests {
    use crate::router::Router;
    use test_case::test_case;
    use tokio::time::Duration;

    #[test_case(200, 150, true)]
    #[test_case(200, 250, false)]
    #[tokio::test]
    async fn test_endpoint_and_router_with_timeout(
        timeout_in_msecs: u64,
        response_delay_in_msecs: u64,
        is_ok: bool,
    ) {
        // Define a timeout and response delay
        let timeout = Duration::from_millis(timeout_in_msecs);
        let response_delay = Duration::from_millis(response_delay_in_msecs);
        // Create a Router
        let router: Router<String, String> = Router::default();

        // Spawn a worker to handle requests
        let (request_receiver, response_sender) = router.request_response_channels();
        tokio::spawn(async move {
            while let Ok((uuid, request)) = request_receiver.recv().await {
                tokio::time::sleep(response_delay).await;
                let response = format!("Response to request: {}", request);
                response_sender.send((uuid, response)).await.unwrap();
            }
        });
        // Spawn the router
        router.tokio_spawn();

        // Make a request to the endpoint
        let request = "Hello, world!".to_string();
        let response = router.endpoint(Some(timeout)).handle_request(request).await;
        assert_eq!(response.is_ok(), is_ok);
    }
}
