//! # Klipper Configuration Compatibility Layer
//!
//! This library provides tools for parsing Klipper `printer.cfg` files and
//! migrating their settings into strongly-typed Rust structs. It aims to
//! ease the transition for host software that needs to understand Klipper
//! configurations, providing warnings and automated migrations where possible.
//!
//! The main entry point is the `migrate_config` function, which takes the
//! raw content of a `printer.cfg` file and returns a structured `PrinterConfig`
//! along with a `MigrationReport`.
//!
//! ## Example Usage
//!
//! ```no_run
//! use std::fs;
//! use compat_layer::migrate_config;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config_content = fs::read_to_string("path/to/printer.cfg")?;
//!     let (config, report) = migrate_config(&config_content)?;
//!
//!     println!("Successfully migrated config!");
//!     println!("Kinematics: {}", config.kinematics);
//!
//!     if !report.warnings.is_empty() {
//!         println!("\nMigration Warnings:");
//!         for warning in report.warnings {
//!             println!("- {}", warning);
//!         }
//!     }
//!
//!     // You can then serialize the typed config to JSON or another format.
//!     let json_output = serde_json::to_string_pretty(&config)?;
//!     println!("\nJSON representation:\n{}", json_output);
//!
//!     Ok(())
//! }
//! ```

pub mod errors;
pub mod migrator;
pub mod models;
pub mod parser;

pub use errors::MigrationError;
pub use migrator::{migrate_config, MigrationReport};
pub use models::*;
