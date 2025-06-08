use chrono::{
    DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, ParseError, TimeZone, Timelike, Utc,
};

fn print_date_parts() {
    let now = Utc::now();

    let (is_pm, hour) = now.hour12();
    println!(
        "TimeUTC: {:02}:{:02}:{:02} {}",
        hour,
        now.minute(),
        now.second(),
        if is_pm { "PM" } else { "AM" }
    );
    println!("Seconds from midnight: {}", now.num_seconds_from_midnight());

    let (is_common_era, year) = now.year_ce();

    println!(
        "DateUTC: {}-{:02}-{:02} {:?} ({})",
        year,
        now.month(),
        now.day(),
        now.weekday(),
        if is_common_era { "CE" } else { "BCE" }
    );
    println!("Days from the Common Era: {}", now.num_days_from_ce());
}

fn to_unix() {
    let start_unix = Utc.timestamp_opt(0, 0).single().unwrap();
    let date_time = NaiveDate::from_ymd_opt(2025, 06, 01)
        .unwrap()
        .and_hms_opt(15, 30, 23)
        .unwrap();
    println!(
        "Seconds between {start_unix} and {} is {}.",
        date_time,
        date_time.and_utc().timestamp()
    );
    let date_time_after_a_billion_seconds = DateTime::from_timestamp(1_000_000_000, 0).unwrap();
    println!(
        "Date after a billion seconds since {start_unix} was {}.",
        date_time_after_a_billion_seconds
    );
}

fn print_formatted() {
    let now = Utc::now();
    println!("UTC now is: {}", now);
    println!("UTC now is: {}", now.to_rfc2822());
    println!("UTC now is: {}", now.to_rfc3339());
    println!("UTC now is: {}", now.format("%a %b %e %T %Y"));
}

fn parse() -> Result<(), ParseError> {
    let dt = DateTime::parse_from_rfc3339("1993-01-23T16:39:57-08:00")?;
    println!("Parsed: {}", dt);
    let dt = DateTime::parse_from_str("12:01:00 19.03.2000 +0000", "%H:%M:%S %d.%m.%Y %z")?;
    println!("Parsed: {}", dt);

    let dt = NaiveTime::parse_from_str("12:01:00", "%H:%M:%S")?;
    println!("Parsed: {}", dt);
    let dt = NaiveDate::parse_from_str("19.03.2000", "%d.%m.%Y")?;
    println!("Parsed: {}", dt);
    let dt = NaiveDateTime::parse_from_str("12:01:00 19.03.2000", "%H:%M:%S %d.%m.%Y")?;
    println!("Parsed: {}", dt);
    Ok(())
}

fn main() {
    print_date_parts();
    to_unix();
    print_formatted();
    parse().unwrap();
}
