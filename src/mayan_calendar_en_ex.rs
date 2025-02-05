use chrono::{Datelike, NaiveDate, Utc};

/// Convert a Gregorian date to Julian Day Number (JDN)
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Calculate Year Bearer (Patron Tzolk’in Day of Haab’ New Year)
fn year_bearer(jdn: i32) -> &'static str {
    let tzolkin_days = ["Ik'", "Manik'", "Eb'", "K’an"];
    let year_start_tzolkin_index = (((jdn + 348) % 260) % 4) as usize;
    tzolkin_days[year_start_tzolkin_index]
}

/// Compute the Moon Phase
fn moon_phase(jdn: i32) -> &'static str {
    let synodic_month = 29.530588; // Average lunar cycle
    let moon_age = (jdn as f64 % synodic_month) / synodic_month;
    
    if moon_age < 0.1 {
        "🌑 New Moon"
    } else if moon_age < 0.25 {
        "🌓 First Quarter"
    } else if moon_age < 0.5 {
        "🌕 Full Moon"
    } else if moon_age < 0.75 {
        "🌗 Last Quarter"
    } else {
        "🌑 New Moon"
    }
}

/// Compute Venus Cycle Phase
fn venus_phase(jdn: i32) -> &'static str {
    let venus_cycle = 584; // Venus synodic period in days
    let phase = jdn % venus_cycle;
    
    if phase < 50 {
        "🌟 Morning Star (Heliacal Rise)"
    } else if phase < 215 {
        "☀️ Superior Conjunction (Invisible)"
    } else if phase < 265 {
        "⭐ Evening Star (Heliacal Set)"
    } else {
        "🌑 Inferior Conjunction (Between Earth & Sun)"
    }
}

fn main() {
    // Get today's Gregorian date
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    // Convert to Julian Day Number
    let jdn = gregorian_to_jdn(year, month, day);

    // Calculate Year Bearer, Moon Phase, and Venus Cycle
    let bearer = year_bearer(jdn);
    let moon = moon_phase(jdn);
    let venus = venus_phase(jdn);

    // Display results
    println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("🔢 Julian Day Number: {}", jdn);
    println!("🌞 Year Bearer: {}", bearer);
    println!("🌙 Moon Phase: {}", moon);
    println!("✨ Venus Cycle: {}", venus);
}
