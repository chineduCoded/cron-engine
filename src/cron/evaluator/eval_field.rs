use crate::cron::ir::FieldMatcher;

pub fn matches(
    field: &FieldMatcher,
    value: u32,
) -> bool {
    field.contains(value)
}
