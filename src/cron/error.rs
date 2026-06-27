use thiserror::Error;

#[derive(Debug, Error)]
pub enum CronError {
    #[error("cron pattern is empty")]
    EmptyPattern,

    #[error("invalid date produced by pattern")]
    InvalidDate,

    #[error("invalid time produced by pattern")]
    InvalidTime,

    #[error("time search limit exceeded")]
    TimeSearchLimitExceeded,

    #[error("invalid pattern: {reason}")]
    InvalidPattern { reason: String },

    #[error("illegal characters in pattern: {chars}")]
    IllegalCharacters { chars: String },

    #[error("component error at position {position}: {reason}")]
    ComponentError { position: usize, reason: String },

    #[error("invalid cron expression")]
    InvalidExpression,

    #[error("invalid field count, expected {expected} but found {found}")]
    InvalidFieldCount { expected: usize, found: usize },

    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("invalid day rule")]
    InvalidDayRule(String),

    #[error("value out of bounds")]
    OutOfBounds,
    
    #[error("missing seconds field")]
    MissingSecondsField,

    #[error("unexpected seconds field")]
    UnexpectedSecondsField,

    #[error("missing year field")]
    MissingYearField,

    #[error("unexpected year field")]
    UnexpectedYearField,
}
