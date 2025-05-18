use chrono::{FixedOffset, Utc};

pub fn get_time() -> String {
    let offset = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now()
        .with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
