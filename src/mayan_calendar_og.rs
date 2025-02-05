use chrono::prelude::*;

/// Convert a Gregorian date to Julian Day Number (JDN)
/// using the algorithm from https://en.wikipedia.org/wiki/Julian_day.
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Convert days since creation (Long Count epoch) to a Long Count date.
/// The Long Count is represented as baktun.katun.tun.uinal.kin.
/// 1 baktun = 144,000 days, 1 katun = 7,200 days, 1 tun = 360 days,
/// 1 uinal = 20 days, and 1 kin = 1 day.
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

/// The Tzolkin date consists of a number (1–13) and a day name (one of 20).
/// Commonly, the correlation is chosen so that 0.0.0.0.0 corresponds to 4 Ahau.
/// We use the formulas:
///   Tzolkin number = ((days + 3) mod 13) + 1
///   Tzolkin name = names[(days + 19) mod 20]
struct TzolkinDate {
    number: i32,
    name: &'static str,
}

fn tzolkin_date(days: i32) -> TzolkinDate {
    let number = (((days + 3) % 13 + 13) % 13) + 1; // ensuring non-negative mod
    let names = [
        "Imix", "Ik", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik", "Lamat", "Muluc", "Oc",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Cib", "Caban", "Etznab", "Kawak", "Ahau"
    ];
    let name = names[(((days + 19) % 20 + 20) % 20) as usize];
    TzolkinDate { number, name }
}

/// The Haab date is a 365‑day solar calendar with 18 months of 20 days and one short month (Uayet) of 5 days.
/// One common correlation is that 0.0.0.0.0 corresponds to 8 Cumku. Cumku is the 18th month (index 17)
/// and day 8. That makes its day-of-year number 17*20 + 8 = 348.
/// We then compute:
///   haab_day = (days + 348) mod 365,
/// and let the Haab month be haab_day / 20, and the day within the month be haab_day % 20.
struct HaabDate {
    day: i32,
    month: &'static str,
}

fn haab_date(days: i32) -> HaabDate {
    let haab_day = ((days + 348) % 365 + 365) % 365; // ensure non-negative result
    let month_index = haab_day / 20;
    let day = haab_day % 20;
    let months = [
        "Pop", "Wo", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
        "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
        "Kayab", "Cumku", "Uayet"
    ];
    // For Uayet, which is the 19th element, days will be in the range 0-4.
    let month = months[month_index as usize];
    HaabDate { day, month }
}

fn main() {
    // Get the current UTC date.
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    // Convert the current Gregorian date to JDN.
    let jdn = gregorian_to_jdn(year, month, day);
    // GMT correlation: 0.0.0.0.0 corresponds to JDN 584283.
    let days_since_creation = jdn - 584283;

    // Compute Long Count.
    let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);

    // Compute Tzolkin date.
    let tzolkin = tzolkin_date(days_since_creation);
    // Compute Haab date.
    let haab = haab_date(days_since_creation);

    println!("Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("Julian Day Number: {}", jdn);
    println!("Days since Mayan creation (0.0.0.0.0): {}", days_since_creation);
    println!("Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin);
    println!("Tzolkin Date: {} {}", tzolkin.number, tzolkin.name);
    println!("Haab Date: {} {}", haab.day, haab.month);
}
