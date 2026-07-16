//! # cron-engine
//!
//! `cron-engine` is a high-performance, Quartz-compatible cron scheduler
//! written in Rust.
//!
//! The library parses cron expressions into an optimized intermediate
//! representation (IR) and computes matching occurrences with minimal
//! allocations.
//!
//! ## Features
//!
//! - Quartz-compatible syntax
//! - Optional seconds field
//! - Optional year field
//! - Timezone-aware scheduling
//! - Daylight Saving Time (DST) aware
//! - Zero-allocation scheduling hot path
//! - Efficient bitfield matching
//! - Supports:
//!   - `*`
//!   - lists
//!   - ranges
//!   - steps
//!   - month names
//!   - weekday names
//!   - `L`
//!   - `LW`
//!   - `W`
//!   - `#`
//!   - `5L`
//!
//! ## Example
//!
//! ```
//! use chrono::{TimeZone, Timelike};
//! use chrono_tz::UTC;
//! use cron_engine::cron::CronSchedule;
//!
//! let schedule = CronSchedule::parse("0 */15 * * * *").unwrap();
//!
//! let start = UTC
//!     .with_ymd_and_hms(2025,1,1,0,0,0)
//!     .unwrap();
//!
//! let next = schedule.next_after(start).unwrap();
//!
//! assert_eq!(next.minute(), 15);
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Expression
//!      │
//!      ▼
//! Parser
//!      │
//!      ▼
//! AST
//!      │
//!      ▼
//! Compiler
//!      │
//!      ▼
//! IR (BitFields)
//!      │
//!      ▼
//! Scheduler
//!      │
//!      ▼
//! Iterator / next_after()
//! ```
//!
//! ## Performance
//!
//! - Bitfield matching in O(1)
//! - Calendar calculations without heap allocation
//! - Scheduler hot path performs no heap allocations
//! - Property-tested
//! - Benchmarked using Criterion

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]

pub mod cron;

pub use cron::CronError;
pub use cron::CronSchedule;
