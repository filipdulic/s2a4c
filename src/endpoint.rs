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
use async_channel::{bounded, RecvError, SendError, Sender};
use thiserror::Error;
use tokio::time::{error::Elapsed, timeout};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EndpointError {
    #[error("Error sending request")]
    RequestSend,
    #[error("Error receiving response")]
    ResponseReceive(#[from] RecvError),
    #[error("Request timed out")]
    Timeout(#[from] Elapsed),
}

impl<Request, Response> From<SendError<(Request, Sender<Response>)>> for EndpointError {
    fn from(_: SendError<(Request, Sender<Response>)>) -> Self {
        EndpointError::RequestSend
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
