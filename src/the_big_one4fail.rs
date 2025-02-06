use chrono::{Datelike, NaiveDate, Utc};
use eframe::egui::{CentralPanel, ColorImage, Context, TextureOptions, Ui};
use eframe::App;
use std::collections::HashMap;

/// Convert a Gregorian date to Julian Day Number (JDN)
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
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
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan", "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men", "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw",
    ];
    let kiche_names = [
        "Imox", "Iq'", "Aq'ab'al", "K'at", "Kan", "Kame", "Kej", "Q'anil", "Tojil", "Tz'i'",
        "B'atz'", "E", "Aj", "Ix", "Tz'ikin", "Ajmaq", "No'j", "Tijax", "Kawoq", "Ajpu",
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
        "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol", "Ch'en", "Yax", "Zac",
        "Ceh", "Mac", "Kankin", "Muan", "Pax", "Kayab", "Kumk'u", "Wayeb'",
    ];

    let kiche_months = [
        "Pop", "Wo'", "Sip", "Zotz'", "Tzek", "Xul", "Yaxkin", "Mol", "Chen", "Yax", "Zac",
        "Keh", "Mak", "Kank'in", "Muwan", "Pax", "Kayab", "Kumk'u", "Wayeb'",
    ];

    HaabDate {
        day,
        yucatec_month: yucatec_months[month_index as usize],
        kiche_month: kiche_months[month_index as usize],
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

/// Moon Phase Calculation
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

/// Load Glyph Images
fn get_tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("Ajaw", "C:/users/phine/desktop/tzolk'in/glyphs/ajaw.png");
    // Add other glyphs as needed
    glyphs
}

/// UI Rendering
fn ui_example(ui: &mut Ui, ctx: &Context) {
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    let jdn = gregorian_to_jdn(year, month, day);
    let days_since_creation = jdn - 584283;

    let tzolkin = tzolkin_date(days_since_creation);
    let haab = haab_date(days_since_creation);
    let moon = moon_phase(jdn);

    let glyphs = get_tzolkin_glyphs();
    ui.vertical(|ui| {
        ui.label("Mayan Date:");
        ui.label(format!("📅 Gregorian Date: {}-{:02}-{:02}", year, month, day));
        ui.label(format!("🌞 Tzolk'in Date: {} {}", tzolkin.number, tzolkin.yucatec_name));
        ui.label(format!("🌙 Haab’ Date: {} {}", haab.day, haab.yucatec_month));
        ui.label(format!("🌕 Moon Phase: {}", moon));

        if let Some(image_path) = glyphs.get(tzolkin.yucatec_name) {
            match load_image_as_texture(ctx, image_path) {
                Ok(texture) => {
                    ui.image(&texture);
                }
                Err(err) => {
                    ui.label(format!("❌ Failed to load image: {}", err));
                }
            }
        } else {
            ui.label("❌ No glyph found!");
        }
    });
}

/// App Struct
struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui_example(ui, ctx);
        });
    }
}

/// Main Function
fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp))),
    ) {
        eprintln!("Failed to launch app: {}", e);
    }
}
