//! Stepflow JSON-RPC Framework
//! 
//! A simple, efficient JSON-RPC 2.0 implementation over TCP for microservices communication.
//! Designed for single-machine deployment with high performance and ease of use.

pub mod protocol;
pub mod server;
pub mod client;
pub mod registry;
pub mod error;
pub mod event;
pub mod streaming;
pub mod subscription_manager;

pub use protocol::*;
pub use server::*;
pub use client::*;
pub use registry::*;
pub use error::*;
pub use event::*;
pub use streaming::*;
pub use subscription_manager::*; 