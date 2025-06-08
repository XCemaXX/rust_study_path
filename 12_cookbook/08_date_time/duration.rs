use std::{thread, time::Instant};

use chrono::{DateTime, FixedOffset, Local, TimeDelta, Utc};

fn expensive_function() {
    thread::sleep(std::time::Duration::from_millis(200));
}

fn measure_duration() {
    let start = Instant::now();
    expensive_function();
    let duration = start.elapsed();
    println!("Elapsed: {:?}", duration);
}

fn day_earlier(dt: DateTime<Utc>) -> Option<DateTime<Utc>> {
    dt.checked_sub_signed(chrono::Duration::days(1))
}

fn add_weeks() {
    let now = Utc::now();
    println!("{}", now);

    let almost_3weeks_from_now = now
        .checked_add_signed(chrono::Duration::weeks(2))
        .and_then(|in_2weeks| in_2weeks.checked_add_signed(chrono::Duration::weeks(1)))
        .and_then(day_earlier);

    match almost_3weeks_from_now {
        Some(d) => println!("{d}"),
        None => eprintln!("Almost three weeks from now overflows!"),
    }

    match now.checked_add_signed(TimeDelta::MAX) {
        Some(x) => println!("{}", x),
        None => eprintln!(
            "We can't use chrono to tell the time for the Solar System to complete more than one full orbit around the galactic center."
        ),
    }
}

fn convert_timezone() {
    let local_time = Local::now();
    let utc_time = DateTime::<Utc>::from_naive_utc_and_offset(local_time.naive_utc(), Utc);
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    let rio_timezone = FixedOffset::west_opt(2 * 3600).unwrap();
    println!("Local time now is {}", local_time);
    println!("UTC time now is {}", utc_time);
    println!(
        "Time in Hong Kong now is {}",
        utc_time.with_timezone(&china_timezone)
    );
    println!(
        "Time in Rio de Janeiro now is {}",
        utc_time.with_timezone(&rio_timezone)
    );
}

fn main() {
    measure_duration();
    add_weeks();
    convert_timezone();
}
