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
//!     let endpoint = Endpoint::new(sender, Some(Duration::from_secs(5)));
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
