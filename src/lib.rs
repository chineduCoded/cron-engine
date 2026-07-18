//! # cron-engine
//!
//! A high-performance, Quartz-compatible cron parser and scheduler.
//!
//! `cron-engine` parses cron expressions into an optimized intermediate
//! representation (IR) and efficiently computes matching occurrences.
//!
//! ## Quick Start
//!
//! ```
//! use chrono::{TimeZone, Timelike};
//! use chrono_tz::UTC;
//! use cron_engine::CronSchedule;
//!
//! let schedule = CronSchedule::parse("0 */15 * * * *").unwrap();
//!
//! let start = UTC
//!     .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
//!     .unwrap();
//!
//! let next = schedule.next_after(start).unwrap();
//!
//! assert_eq!(next.minute(), 15);
//! ```
//!
//! ## Features
//!
//! - Quartz-compatible syntax
//! - Time zone and DST aware
//! - Optional seconds and year fields
//! - Efficient bitfield matching
//! - Advanced Quartz day rules (`L`, `LW`, `W`, `#`, `5L`)
//! - Lazy occurrence iterators
//! - Zero-allocation scheduler hot path
//!
//! ## Main Types
//!
//! - [`CronSchedule`] — immutable compiled schedule.
//! - [`CronError`] — parsing and scheduling errors.
//!
//! ## Architecture
//!
//! ```text
//! Expression
//!     │
//!     ▼
//! Parser
//!     ▼
//! AST
//!     ▼
//! Compiler
//!     ▼
//! IR
//!     ▼
//! Scheduler
//! ```
//!
//! See the `cron` module for implementation details.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]

pub mod cron;

pub use cron::CronError;
pub use cron::CronSchedule;
pub use cron::field::{BitField, CronFlags, LAST_BIT};
