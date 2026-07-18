/// Fields that participate in schedule navigation. 
/// 
/// The scheduler processes these fields from largest (`Year`) to 
/// smallest (`Second`) while searching for the next or previous 
/// matching occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// Calendar year.
    Year,

    /// Month of year (`1..=12`).
    Month,

    /// Day of the month (`1..=31`).
    Day,

    /// Hour of the day (`0..=23`).
    Hour,

    /// Minute of the hour (`0..=59`).
    Minute,

    /// Second of the minute (`0..=59`).
    Second,
}

/// Numeric cron fields backed by a [`FieldMatcher`]. 
///
/// Unlike [`Field`], this enum excludes the day field because day 
/// matching is governed by Quartz calendar rules (`L`, `W`, `LW`, 
/// `#`, etc.) rather than simple numeric matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericField {
    /// Calendar year.
    Year,

    /// Month of the year.
    Month,

    /// Hour of the day.
    Hour,

    /// Minute of the hour.
    Minute,

    /// Second of the minute.
    Second,
}
