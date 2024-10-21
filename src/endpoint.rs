//! # Endpoint Module
//!
//! This module provides the `Endpoint` struct and the `EndpointError` enum for handling asynchronous
//! communication with a timeout mechanism.
//!
//! ## Overview
//!
//! The `Endpoint` struct is designed to facilitate sending requests and receiving responses asynchronously
//! with an optional timeout. It uses channels to send requests and receive responses, ensuring that the
//! communication is non-blocking and efficient.
//!
//! The `EndpointError` enum defines various errors that can occur during the operation of an `Endpoint`,
//! including errors related to sending requests, receiving responses, and timeouts.
//!
//! ## Usage
//!
//! To use the `Endpoint` struct, you need to create an instance of it by providing a sender channel for
//! registration and an optional timeout interval. You can then use this instance to send requests and
//! receive responses asynchronously.
//!
//! ```rust
//! use async_channel::{bounded, Sender};
//! use tokio::time::Duration;
//! use endpoint::{Endpoint, EndpointError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), EndpointError> {
//!     let (sender, receiver) = bounded(100);
//!     let endpoint = Endpoint::new(sender, Some(Duration::from_secs(5)));
//!
//!     // Example usage of the endpoint
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The `EndpointError` enum provides comprehensive error handling for the `Endpoint` struct. It includes
//! variants for request sending errors, response receiving errors, and timeout errors. These errors can
//! be easily converted from the corresponding error types in the `async_channel` and `tokio` crates.
//!
//! ```rust
//! use endpoint::EndpointError;
//!
//! fn handle_error(error: EndpointError) {
//!     match error {
//!         EndpointError::RequestSendError => {
//!             eprintln!("Failed to send request");
//!         }
//!         EndpointError::ResponseReceiveError(_) => {
//!             eprintln!("Failed to receive response");
//!         }
//!         EndpointError::TimeoutError(_) => {
//!             eprintln!("Request timed out");
//!         }
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - Asynchronous request-response handling
//! - Optional timeout for requests
//! - Comprehensive error handling with `EndpointError`
//!
//! ## Dependencies
//!
//! - `async_channel` for asynchronous message passing
//! - `tokio` for handling timeouts
//! - `thiserror` for error handling

use async_channel::{bounded, RecvError, SendError, Sender};
use thiserror::Error;
use tokio::time::{error::Elapsed, timeout};

#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("Error sending request")]
    RequestSendError,
    #[error("Error receiving response")]
    ResponseReceiveError(#[from] RecvError),
    #[error("Request timed out")]
    TimeoutError(#[from] Elapsed),
}

impl<Request, Response> From<SendError<(Request, Sender<Response>)>> for EndpointError {
    fn from(_: SendError<(Request, Sender<Response>)>) -> Self {
        EndpointError::RequestSendError
    }
}

pub struct Endpoint<Request, Response> {
    registration_sender: Sender<(Request, Sender<Response>)>,
    timeout_interval: Option<std::time::Duration>,
}

impl<Request, Response> Endpoint<Request, Response>
where
    Request: Send + 'static,
    Response: Send + 'static,
{
    pub fn new(
        registration_sender: Sender<(Request, Sender<Response>)>,
        timeout_interval: Option<std::time::Duration>,
    ) -> Self {
        Self {
            registration_sender,
            timeout_interval,
        }
    }
    pub async fn handle_request(&self, request: Request) -> Result<Response, EndpointError> {
        let (response_sender, response_receiver) = bounded(100);
        let registration_sender = self.registration_sender.clone();
        registration_sender.send((request, response_sender)).await?;
        let response = match self.timeout_interval {
            Some(interval) => timeout(interval, response_receiver.recv()).await?,
            None => response_receiver.recv().await,
        };
        response.map_err(|e| e.into())
    }
}
