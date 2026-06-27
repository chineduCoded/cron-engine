use chrono::{
    DateTime,
    NaiveDateTime,
    TimeZone,
};

pub fn resolve_local<Tz>(
    tz: &Tz,
    naive: NaiveDateTime,
) -> Option<DateTime<Tz>>
where 
    Tz: TimeZone,
{
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => {
            Some(dt)
        }

        chrono::LocalResult::Ambiguous(
            earlier, 
            _,
        ) => Some(earlier),

        chrono::LocalResult::None => None,
    }
}











