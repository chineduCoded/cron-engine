#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldExpr {
    #[default]
    Wildcard,

    List(Vec<FieldExpr>),
    Range(u32, u32),
    Step(Box<FieldExpr>, u32),
    Value(u32),
    LastDay,                           // L
    LastWeekday(u32), // e.g. 5L -> last weekday '5' (weekday 0..6)
    LastBusinessDay,
    NthWeekday { weekday: u32, nth: u32 }, // e.g. 5#3 -> 3rd weekday '5' (weekday 0..6)
    NearestWeekday(u32),               // e.g. 15W -> nearest weekday to 15th
    And(Box<FieldExpr>, Box<FieldExpr>),// a + b -> intersection (dom AND dow)
}

/// Parsed cron expression.
///
/// Example:
///
/// "*/5 9-17 * * MON-FRI"
///
#[derive(Debug, Clone)]
pub struct CronAst {
    pub second: FieldExpr,
    pub minute: FieldExpr,
    pub hour: FieldExpr,

    pub day_of_month: FieldExpr,
    pub month: FieldExpr,
    pub day_of_week: FieldExpr,

    pub year: Option<FieldExpr>,

    /// true => DOM OR DOW
    /// false => DOM AND DOW
    pub dom_dow_or: bool,
}
