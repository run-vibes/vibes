//! Middleware for the vibes server

mod auth;

pub use auth::{auth_middleware, AuthLayer};
