use chrono::{Datelike, NaiveDate, Utc};
use std::collections::HashMap;

/// Convert a Gregorian date to Julian Day Number (JDN)
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Convert days since Mayan creation to Long Count format
fn long_count(days: i32) -> (i32, i32, i32, i32, i32) {
    let baktun = days / 144_000;
    let rem1 = days % 144_000;
    let katun = rem1 / 7_200;
    let rem2 = rem1 % 7_200;
    let tun = rem2 / 360;
    let rem3 = rem2 % 360;
    let uinal = rem3 / 20;
    let kin = rem3 % 20;
    (baktun, katun, tun, uinal, kin)
}

/// Convert a number (0-19) to a Mayan numeral Unicode character
fn mayan_numeral(n: i32) -> char {
    match n {
        0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap(),
        _ => '❓', // If out of range, return a placeholder
    }
}

/// Find a historical Mayan event for the given JDN
fn historical_event(jdn: i32) -> Option<&'static str> {
    let events = [
        (-3113, 8, 11, "🌎 The Maya creation date (0.0.0.0.0)"),
        (292, 1, 1, "📜 Earliest Long Count Date Found"),
        (378, 1, 16, "⚔️ Teotihuacan Influence Over Tikal Begins"),
        (682, 6, 3, "👑 King Jasaw Chan K’awiil I Crowned in Tikal"),
        (1511, 8, 1, "⚔️ Spanish Make First Contact with the Maya"),
        (1697, 3, 13, "🏹 Spanish Conquer the Last Maya City, Tayasal"),
    ];

    for (e_year, e_month, e_day, desc) in events.iter() {
        let e_jdn = gregorian_to_jdn(*e_year, *e_month, *e_day);
        if jdn == e_jdn {
            return Some(desc);
        }
    }

    None
}

fn main() {
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    let jdn = gregorian_to_jdn(year, month, day);
    let days_since_creation = jdn - 584283; // Days since 0.0.0.0.0

    let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);

    println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("🔢 Julian Day Number: {}", jdn);
    println!("📜 Long Count: {}.{}.{}.{}.{}  {}{}{}{}{}",
        baktun, katun, tun, uinal, kin,
        mayan_numeral(baktun), mayan_numeral(katun), mayan_numeral(tun), mayan_numeral(uinal), mayan_numeral(kin)
    );

    if let Some(event) = historical_event(jdn) {
        println!("🏛️ Historical Event Today: {}", event);
    }
}
