#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericField {
    Year,
    Month,
    Hour,
    Minute,
    Second,
}
