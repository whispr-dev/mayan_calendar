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
    let glyphs = long_count_glyphs();

    println!("📜 Long Count: {}.{}.{}.{}.{}  {}{}{}{}{}",
        baktun, katun, tun, uinal, kin,
        glyphs[&baktun], glyphs[&katun], glyphs[&tun], glyphs[&uinal], glyphs[&kin]);

    if let Some(event) = historical_event(jdn) {
        println!("🏛️ Historical Event Today: {} {}", event, historical_glyphs()[event]);
    }
}
