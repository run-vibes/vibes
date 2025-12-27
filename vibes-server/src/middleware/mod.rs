//! Middleware for the vibes server

mod auth;

pub use auth::{AuthLayer, auth_middleware};
