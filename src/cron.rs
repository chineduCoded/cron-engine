//! A production-ready cron expression parser and scheduler.
//!
//! `cron-engine` provides:
//!
//! - Quartz-style cron expression parsing
//! - Efficient bitset-based schedule evaluation
//! - Time zone aware scheduling
//! - DST-safe occurrence calculation
//! - Support for advanced day rules (`L`, `W`, `LW`, `#`)
//! - Forward and backward schedule navigation
//!
//! The primary entry point is [`CronSchedule`].

/// Abstract syntax tree for parsed cron expressions.
pub mod ast;

/// Compiles parsed expressions into an optimized intermediate representation.
pub mod compiler;

/// Error types returned by parsing, compilation, and scheduling.
pub mod error;

/// Schedule evaluation utilities.
pub mod evaluator;

/// Bitfield and field matcher implementations.
pub mod field;

/// Optimized intermediate representation used by the scheduler.
pub mod ir;

/// Cron expression parser.
pub mod parser;

/// Lookup tables for named months and weekdays.
pub mod resolver;

/// Scheduler implementation and iterators.
pub mod scheduler;

/// Time zone conversion helpers.
pub mod timezone;

/// Common error type returned throughout the crate.
pub use error::CronError;

/// High-level immutable cron schedule.
pub use scheduler::scheduler::CronSchedule;
