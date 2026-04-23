pub mod config;
pub mod application;
pub mod infrastructure;
pub mod interface;
pub mod shared;

// Re-export domain from infrastructure
pub use infrastructure::domain;
