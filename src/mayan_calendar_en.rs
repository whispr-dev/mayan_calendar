use chrono::prelude::*;

/// Convert a Gregorian date to Julian Day Number (JDN)
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Convert days since Mayan creation to Long Count
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

/// Tzolk'in Calendar: Yucatec vs. K’iche’ names
struct TzolkinDate {
    number: i32,
    yucatec_name: &'static str,
    kiche_name: &'static str,
}

fn tzolkin_date(days: i32) -> TzolkinDate {
    let number = (((days + 3) % 13 + 13) % 13) + 1;
    let yucatec_names = [
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
    ];
    let kiche_names = [
        "Imox", "Iq'", "Aq'ab'al", "K'at", "Kan",
        "Kame", "Kej", "Q'anil", "Tojil", "Tz'i'",
        "B'atz'", "E", "Aj", "Ix", "Tz'ikin",
        "Ajmaq", "No'j", "Tijax", "Kawoq", "Ajpu"
    ];
    let index = (((days + 19) % 20 + 20) % 20) as usize;
    TzolkinDate {
        number,
        yucatec_name: yucatec_names[index],
        kiche_name: kiche_names[index],
    }
}

/// Haab’ Calendar: Yucatec vs. K’iche’ names
struct HaabDate {
    day: i32,
    yucatec_month: &'static str,
    kiche_month: &'static str,
}

fn haab_date(days: i32) -> HaabDate {
    let haab_day = ((days + 348) % 365 + 365) % 365;
    let month_index = haab_day / 20;
    let day = haab_day % 20;
    
    let yucatec_months = [
        "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
        "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
        "Kayab", "Kumk'u", "Wayeb'"
    ];
    
    let kiche_months = [
        "Pop", "Wo'", "Sip", "Zotz'", "Tzek", "Xul", "Yaxkin", "Mol",
        "Chen", "Yax", "Zac", "Keh", "Mak", "Kank'in", "Muwan", "Pax",
        "Kayab", "Kumk'u", "Wayeb'"
    ];
    
    let month = yucatec_months[month_index as usize];
    let kiche_month = kiche_months[month_index as usize];

    HaabDate {
        day,
        yucatec_month: month,
        kiche_month,
    }
}

fn main() {
    // Get the current UTC date.
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    // Convert to Julian Day Number (JDN)
    let jdn = gregorian_to_jdn(year, month, day);
    let days_since_creation = jdn - 584283;

    // Compute Mayan Calendar Dates
    let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);
    let tzolkin = tzolkin_date(days_since_creation);
    let haab = haab_date(days_since_creation);

    // Print results
    println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("🔢 Julian Day Number: {}", jdn);
    println!("🕰 Days since 0.0.0.0.0: {}", days_since_creation);
    println!("📜 Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin);
    println!("🌞 Tzolk'in Date: {} {} (K’iche’: {})", tzolkin.number, tzolkin.yucatec_name, tzolkin.kiche_name);
    println!("🌙 Haab’ Date: {} {} (K’iche’: {})", haab.day, haab.yucatec_month, haab.kiche_month);
}
