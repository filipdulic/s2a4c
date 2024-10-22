//! # Router Module
//!
//! This module provides the `Router` struct for managing asynchronous request-response communication
//! using unique identifiers (UUIDs). It leverages the `async_channel` crate for message passing and
//! the `tokio` crate for asynchronous operations.
//!
//! ## Overview
//!
//! The `Router` struct is designed to facilitate the routing of requests and responses between different
//! components of an application. It uses channels to send and receive requests and responses, ensuring
//! that the communication is non-blocking and efficient. Each request is associated with a unique UUID
//! to match it with the corresponding response.
//!
//! The `Router` struct also provides a default implementation for easy instantiation with pre-configured
//! channel capacities.
//!
//! ## Usage
//!
//! To use the `Router` struct, you need to create an instance of it, either using the default implementation
//! or by manually configuring the channels. You can then use this instance to send requests and receive
//! responses asynchronously.
//!
//! ```rust
//! use async_channel::{bounded, Sender, Receiver};
//! use uuid::Uuid;
//! use std::collections::HashMap;
//! use sync2async4coms::router::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let router: Router<String, String> = Router::default();
//!
//!     // Example usage of the router
//!     // ...
//! }
//! ```
//!
//! ## Features
//!
//! - Asynchronous request-response routing
//! - Unique identifier (UUID) based matching
//! - Default implementation for easy instantiation
//!
//! ## Dependencies
//!
//! - `async_channel` for asynchronous message passing
//! - `tokio` for asynchronous operations
//! - `uuid` for generating unique identifiers
//! - `std::collections::HashMap` for mapping UUIDs to response senders
use std::{future::Future, sync::Arc, time::Duration};

use async_channel::{bounded, Receiver, Sender};
use scc::HashMap;
use uuid::Uuid;

use crate::endpoint::Endpoint;

#[derive(Debug, Clone)]
pub struct Router<Request, Response> {
    registration_sender: Sender<(Request, Sender<Response>)>,
    registration_receiver: Receiver<(Request, Sender<Response>)>,
    request_sender: Sender<(Uuid, Request)>,
    request_receiver: Receiver<(Uuid, Request)>,
    response_receiver: Receiver<(Uuid, Response)>,
    response_sender: Sender<(Uuid, Response)>,
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
}

impl<Request, Response> Default for Router<Request, Response> {
    fn default() -> Self {
        let (registration_sender, registration_receiver) = bounded(100);
        let (request_sender, request_receiver) = bounded(100);
        let (response_sender, response_receiver) = bounded(100);
        Self {
            registration_sender,
            registration_receiver,
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
            response_map: Arc::new(HashMap::new()),
        }
    }
}

async fn response_loop<Response>(
    response_receiver: Receiver<(Uuid, Response)>,
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
) where
    Response: Send + 'static + Clone,
{
    while let Ok((uuid, response)) = response_receiver.recv().await {
        match response_map.remove_async(&uuid).await {
            Some((_, sender)) => match sender.send(response).await {
                //TODO: Handle error via logging and tracing
                Ok(_) => {
                    println!("Success from resp loop")
                }
                Err(err) => {
                    println!("Error from resp loop : {:?}", err)
                }
            },
            None => {
                println!(
                    "Error from resp loop : No sender found for uuid: {:?}",
                    uuid
                );
            }
        }
    }
}

async fn registration_loop<Request, Response>(
    registration_receiver: Receiver<(Request, Sender<Response>)>,
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
    request_sender: Sender<(Uuid, Request)>,
) where
    Request: Send + 'static + Clone,
    Response: Send + 'static + Clone,
{
    while let Ok((request, response_sink)) = registration_receiver.recv().await {
        // insert can fail if key already exists, unlikly but handled.
        let mut uuid = Uuid::new_v4();
        while response_map
            .insert_async(uuid, response_sink.clone())
            .await
            //.await
            .is_err()
        {
            uuid = Uuid::new_v4();
        }
        let request_sender = request_sender.clone();
        tokio::spawn(async move {
            //TODO: Handle error via logging and tracing
            match request_sender.send((uuid, request)).await {
                Ok(_) => {
                    println!("Success from reg loop")
                }
                Err(err) => {
                    println!("Error from reg loop : {:?}", err)
                }
            };
        });
    }
}

impl<Request, Response> Router<Request, Response>
where
    Request: Send + 'static + Clone,
    Response: Send + 'static + Clone,
{
    pub fn endpoint(&self, timeout: Option<Duration>) -> Endpoint<Request, Response> {
        Endpoint::new(self.registration_sender.clone(), timeout)
    }
    #[allow(clippy::type_complexity)]
    pub fn request_response_channels(
        &self,
    ) -> (Receiver<(Uuid, Request)>, Sender<(Uuid, Response)>) {
        (self.request_receiver.clone(), self.response_sender.clone())
    }

    pub fn tokio_spawn(&self) -> tokio::task::JoinHandle<()> {
        let temp = self.clone();
        tokio::spawn(async move { temp.run().await })
    }

    pub fn tokio_spawn_workers<F>(
        &self,
        num_workers: usize,
        worker_fn: impl Fn(Receiver<(Uuid, Request)>, Sender<(Uuid, Response)>) -> F,
    ) -> Vec<tokio::task::JoinHandle<()>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            handles.push(tokio::spawn(worker_fn(
                self.request_receiver.clone(),
                self.response_sender.clone(),
            )));
        }
        handles
    }

    pub async fn run(&self) {
        let response_loop = tokio::spawn(response_loop(
            self.response_receiver.clone(),
            self.response_map.clone(),
        ));
        let registration_loop = tokio::spawn(registration_loop(
            self.registration_receiver.clone(),
            self.response_map.clone(),
            self.request_sender.clone(),
        ));
        let _ = tokio::join!(response_loop, registration_loop);
    }
}
