/// Abstract syntax tree (AST) node representing a single cron field.
///
/// This is the output of the parser and the input to the compiler.
/// During compilation each variant is transformed into its optimized
/// internal representation.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldExpr {
    /// Matches every valid value.
    ///
    /// Example:
    ///
    /// ```text
    /// *
    /// ```
    #[default]
    Wildcard,

    /// Matches any expression in the list.
    ///
    /// Example:
    ///
    /// ```text
    /// 1,5,10
    /// ```
    List(Vec<FieldExpr>),

    /// Matches every value within an inclusive range.
    ///
    /// Example:
    ///
    /// ```text
    /// 10-20
    /// ```
    Range(u32, u32),

    /// Applies a step interval to another expression.
    ///
    /// Example:
    ///
    /// ```text
    /// */5
    /// 1-30/2
    /// ```
    Step(Box<FieldExpr>, u32),

    /// Matches exactly one value.
    ///
    /// Example:
    ///
    /// ```text
    /// 15
    /// ```
    Value(u32),

    /// Matches the last day of the month (`L`).
    ///
    /// Valid only in the day-of-month field.
    LastDay,

    /// Matches the last occurrence of a weekday in a month.
    ///
    /// The value uses Quartz numbering where Sunday = 0.
    ///
    /// Example:
    ///
    /// ```text
    /// 5L
    /// ```
    /// matches the last Friday of the month.
    LastWeekday(u32),

    /// Matches the last business day of the month (`LW`).
    ///
    /// Business days are Monday through Friday.
    /// If the month's final day falls on a weekend,
    /// the nearest preceding weekday is selected.
    ///
    /// Valid only in the day-of-month field.
    LastBusinessDay,

    /// Matches the _n_th occurrence of a weekday within a month.
    ///
    /// The weekday follows Quartz numbering where Sunday = 0.
    ///
    /// Example:
    ///
    /// ```text
    /// 1#3
    /// ```
    ///
    /// matches the third Monday of the month.
    NthWeekday {
        /// Weekday (0 = Sunday).
        weekday: u32,

        /// occurrence within the month (1-5).
        nth: u32,
    },

    /// Matches the weekday nearest a specific day of the month.
    ///
    /// Example:
    ///
    /// ```text
    /// 15W
    /// ```
    ///
    /// If the 15th falls on Saturday, Friday the 14th matches.
    /// If it falls on Sunday, Monday the 16th matches.
    NearestWeekday(u32),

    /// Logical conjunction of two expressions.
    ///
    /// Primarily used internally during parsing and normalization.
    And(Box<FieldExpr>, Box<FieldExpr>), // a + b -> intersection (dom AND dow)
}

/// Parsed cron expression.
///
/// `CronAst` is produced by the parser before semantic validation and
/// compilation into the optimized [`CronIr`] representation.
///
/// It preserves the original structure of every field expression.
///
/// # Example
///
/// ```text
/// */5 9-17 * * MON-FRI
/// ```
#[derive(Debug, Clone)]
pub struct CronAst {
    /// Seconds field.
    pub second: FieldExpr,

    /// Minutes field.
    pub minute: FieldExpr,

    /// Hours field.
    pub hour: FieldExpr,

    /// Day-of-month field.
    pub day_of_month: FieldExpr,

    /// Month field.
    pub month: FieldExpr,

    /// Day-of-week field.
    pub day_of_week: FieldExpr,

    /// Optional year field.
    ///
    /// When absent, all years are matched.
    pub year: Option<FieldExpr>,

    /// Controls how the day-of-month and day-of-week fields interact.
    ///
    /// - `true`  → day-of-month **OR** day-of-week
    /// - `false` → day-of-month **AND** day-of-week
    pub dom_dow_or: bool,
}
