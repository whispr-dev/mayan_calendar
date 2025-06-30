use chrono::{Datelike, NaiveDate, Utc};
use std::collections::HashMap;

/// Convert a number (0-19) to a Mayan numeral Unicode character
fn mayan_numeral(n: i32) -> char {
    match n {
        0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap(),
        _ => '❓', // Placeholder if out of range
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

/// Get Mayan glyphs for Tzolk’in days
fn tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    let tzolkin_days = [
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
    ];
    let tzolkin_symbols = [
        "🐊", "💨", "🌑", "🌽", "🐍",
        "💀", "🖐️", "🌟", "💧", "🐶",
        "🕷️", "🌾", "🌳", "🦉", "🦅",
        "🐝", "🌀", "🔪", "⛈️", "👑"
    ];
    for i in 0..20 {
        glyphs.insert(tzolkin_days[i], tzolkin_symbols[i]);
    }
    glyphs
}

/// Get Mayan glyphs for Haab’ months
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

fn main() {
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    let tzolkin_glyph_map = tzolkin_glyphs();
    let haab_glyph_map = haab_glyphs();

    // Example: Today's Mayan date values (replace with real calculations)
    let baktun = 13;
    let katun = 0;
    let tun = 12;
    let uinal = 5;
    let kin = 9;

    let tzolkin_name = "Muluk";
    let haab_name = "Pax";

    let tzolkin_glyph = tzolkin_glyph_map.get(tzolkin_name).unwrap_or(&"❓");
    let haab_glyph = haab_glyph_map.get(haab_name).unwrap_or(&"❓");

    println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
    println!("🌞 Tzolk'in Date: {} {} {}", 13, tzolkin_name, tzolkin_glyph);
    println!("🌙 Haab’ Date: {} {} {}", 12, haab_name, haab_glyph);

    println!("📜 Long Count:");
    println!(
        "{}.{}.{}.{}.{}  {}{}{}{}{}",
        baktun, katun, tun, uinal, kin,
        mayan_numeral(baktun), mayan_numeral(katun), mayan_numeral(tun), mayan_numeral(uinal), mayan_numeral(kin)
    );

    println!("\n📜 Long Count (ASCII):");
    println!("Baktun:\n{}", mayan_ascii_number(baktun));
    println!("Katun:\n{}", mayan_ascii_number(katun));
    println!("Tun:\n{}", mayan_ascii_number(tun));
    println!("Uinal:\n{}", mayan_ascii_number(uinal));
    println!("Kin:\n{}", mayan_ascii_number(kin));
}
