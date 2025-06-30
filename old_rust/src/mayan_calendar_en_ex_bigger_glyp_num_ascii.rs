use chrono::{Datelike,  NaiveDate, Utc};
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

/// Generate an ASCII-art Mayan Long Count representation
fn mayan_ascii_number(n: i32) -> String {
    let mut result = String::new();
    let bars = n / 5;
    let dots = n % 5;

    for _ in 0..bars {
        result.push_str("▬\n");
    }
    for _ in 0..dots {
        result.push_str("● ");
    }
    if n == 0 {
        result.push_str("𝋠"); // Mayan zero glyph
    }
    result
}

/// Get Long Count Glyphs
fn long_count_glyphs() -> HashMap<i32, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert(0, "🪵");  // Placeholder for Baktun glyph
    glyphs.insert(1, "🔥");
    glyphs.insert(2, "💧");
    glyphs.insert(3, "🌿");
    glyphs.insert(4, "🌞");
    glyphs.insert(5, "🌕");
    glyphs.insert(6, "🌎");
    glyphs.insert(7, "🐍");
    glyphs.insert(8, "🌪️");
    glyphs.insert(9, "⭐");
    glyphs.insert(10, "🔺");
    glyphs.insert(11, "🏹");
    glyphs.insert(12, "🌀");
    glyphs.insert(13, "🔮");
    glyphs
}

/// Get Tzolk’in Day Glyphs
fn tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    let tzolkin_days = [
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
    ];
    let tzolkin_symbols = [
        "🐊", "🌬️", "🌑", "🌽", "🐍",
        "💀", "🖐️", "🌟", "💧", "🐶",
        "🕷️", "🌾", "🌳", "🦉", "🦅",
        "🐝", "🌀", "🔪", "⛈️", "👑"
    ];
    for i in 0..20 {
        glyphs.insert(tzolkin_days[i], tzolkin_symbols[i]);
    }
    glyphs
}

/// Get Haab’ Month Glyphs
fn haab_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    let haab_months = [
        "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
        "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
        "Kayab", "Kumk'u", "Wayeb'"
    ];
    let haab_symbols = [
        "📜", "🌊", "🔥", "🦇", "🌱", "💨", "🌞", "🌧️",
        "🏺", "🌿", "❄️", "🐆", "🎭", "🔥", "🦜", "🎵",
        "🐢", "🌰", "⚠️"
    ];
    for i in 0..19 {
        glyphs.insert(haab_months[i], haab_symbols[i]);
    }
    glyphs
}

/// Get Historical Event Glyphs
fn historical_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("🌎 The Maya creation date (0.0.0.0.0)", "🌀");
    glyphs.insert("📜 Earliest Long Count Date Found", "📜");
    glyphs.insert("⚔️ Teotihuacan Influence Over Tikal Begins", "⚔️");
    glyphs.insert("🏛️ Dynasty of Copán Founded", "🏛️");
    glyphs.insert("🛑 Tikal Defeated by Calakmul", "🛑");
    glyphs.insert("👑 King Jasaw Chan K’awiil I Crowned in Tikal", "👑");
    glyphs.insert("🏰 Toltec-Maya Rule in Chichen Itzá Begins", "🏰");
    glyphs.insert("🏹 Spanish Conquer the Last Maya City, Tayasal", "🏹");
    glyphs
}

/// Find a historical Mayan event for the given JDN
fn historical_event(jdn: i32) -> Option<&'static str> {
    let events = [
        (-3113, 8, 11, "🌎 The Maya creation date (0.0.0.0.0)"),
        (292, 1, 1, "📜 Earliest Long Count Date Found"),
        (378, 1, 16, "⚔️ Teotihuacan Influence Over Tikal Begins"),
        (426, 1, 1, "🏛️ Dynasty of Copán Founded"),
        (562, 1, 1, "🛑 Tikal Defeated by Calakmul"),
        (682, 6, 3, "👑 King Jasaw Chan K’awiil I Crowned in Tikal"),
        (751, 1, 1, "🏛️ Uxmal Emerges as a Major Power"),
        (869, 12, 1, "🏛️ Tikal Abandoned"),
        (987, 1, 1, "🏰 Toltec-Maya Rule in Chichen Itzá Begins"),
        (1200, 1, 1, "🔺 Decline of Chichen Itzá"),
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

/// Calculate upcoming solstices and equinoxes
fn next_solstice_or_equinox(year: i32, month: i32, day: i32) -> (&'static str, i32) {
    let events = [
        ("🌸 Spring Equinox", NaiveDate::from_ymd_opt(year, 3, 20).unwrap()),
        ("☀️ Summer Solstice", NaiveDate::from_ymd_opt(year, 6, 21).unwrap()),
        ("🍂 Autumn Equinox", NaiveDate::from_ymd_opt(year, 9, 22).unwrap()),
        ("❄️ Winter Solstice", NaiveDate::from_ymd_opt(year, 12, 21).unwrap()),
    ];

    let today = NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();
    
    for (name, date) in events.iter() {
        if *date >= today {
            let days_until = (*date - today).num_days() as i32;
            return (*name, days_until);
        }
    }
    
    // If past December, return next year's Spring Equinox
    ("🌸 Spring Equinox", 365 - (today.ordinal() - 79) as i32)
}

/// Predict next Lunar and Solar Eclipse
fn next_eclipse(jdn: i32) -> &'static str {
    let saros_cycle = 6585; // Average Saros cycle in days (eclipses repeat every ~18 years)
    let days_since_last_eclipse = jdn % saros_cycle;

    if days_since_last_eclipse < 15 {
        "🌑 Lunar Eclipse Soon!"
    } else if days_since_last_eclipse < 30 {
        "🌞 Solar Eclipse Soon!"
    } else {
        "🌘 No Eclipse Imminent"
    }
}

/// Check for Historical Mayan Events
fn historical_events(year: i32, month: i32, day: i32) -> Option<&'static str> {
    let events = [
        (292, 1, 1, "📜 Earliest Long Count Date Found"),
        (378, 1, 16, "🏛️ Teotihuacan Influence Over Tikal Begins"),
        (682, 6, 3, "👑 King Jasaw Chan K’awiil I Crowned in Tikal"),
        (869, 12, 1, "🏛️ Tikal Collapses"),
        (1511, 8, 1, "⚔️ Spanish Make First Contact with the Maya"),
    ];

    for (e_year, e_month, e_day, desc) in events.iter() {
        if *e_year == year && *e_month == month && *e_day == day {
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

    let jdn = gregorian_to_jdn(year, month, day);
    
    let bearer = year_bearer(jdn);
    let moon = moon_phase(jdn);
    let venus = venus_phase(jdn);
    let (solstice, days_until) = next_solstice_or_equinox(year, month, day);
    let eclipse = next_eclipse(jdn);
    let _historical = historical_events(year, month, day);

    // ✅ Fix: Declare glyphs before using it!
    let glyphs = long_count_glyphs();

    println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("🔢 Julian Day Number: {}", jdn);
    println!("🕰 Days since 0.0.0.0.0: {}", days_since_creation);
    println!("🌞 Tzolk'in Date: {} {} (K’iche’: {})", tzolkin.number, tzolkin.yucatec_name, tzolkin.kiche_name);
    println!("🌙 Haab’ Date: {} {} (K’iche’: {})", haab.day, haab.yucatec_month, haab.kiche_month);
    println!("🌞 Year Bearer: {}", bearer);
    println!("🌙 Moon Phase: {}", moon);
    println!("✨ Venus Cycle: {}", venus);
    println!("🌓 Next Solstice/Equinox: {} ({} days away)", solstice, days_until);
    println!("🌘 Eclipse Prediction: {}", eclipse);
    println!("\n📜 Long Count (ASCII):");
    println!("Baktun:\n{}", mayan_ascii_number(baktun));
    println!("Katun:\n{}", mayan_ascii_number(katun));
    println!("Tun:\n{}", mayan_ascii_number(tun));
    println!("Uinal:\n{}", mayan_ascii_number(uinal));
    println!("Kin:\n{}", mayan_ascii_number(kin));
    println!(
      "📜 Long Count: {}.{}.{}.{}.{}  {}{}{}{}{} {}{}{}{}{}",
        baktun, katun, tun, uinal, kin,
        glyphs[&baktun], glyphs[&katun], glyphs[&tun], glyphs[&uinal], glyphs[&kin],
        mayan_numeral(baktun), mayan_numeral(katun), mayan_numeral(tun), 
        mayan_numeral(uinal), mayan_numeral(kin)
    );
        let _ = tzolkin_glyphs();
        let _ = haab_glyphs();
        let _ = historical_glyphs();
    
        if let Some(event) = historical_event(jdn) {
            let event_glyphs = historical_glyphs();
            println!("🏛️ Historical Event Today: {} {}", event, event_glyphs.get(event).unwrap_or(&"❓"));        
    }
}
