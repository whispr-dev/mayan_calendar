use chrono::{Datelike, Utc};
use eframe::egui::{CentralPanel, ColorImage, Context, TextureOptions, Ui};
use eframe::App;
use std::collections::HashMap;

fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

fn tzolkin_date(days: i32) -> (i32, &'static str) {
    let number = (((days + 3) % 13 + 13) % 13) + 1;
    let tzolkin_names = [
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw",
    ];
    let index = (((days + 19) % 20 + 20) % 20) as usize;
    (number, tzolkin_names[index])
}

fn haab_date(days: i32) -> (i32, &'static str) {
    let haab_day = ((days + 348) % 365 + 365) % 365;
    let day = haab_day % 20;
    let month_names = [
        "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
        "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
        "Kayab", "Kumk'u", "Wayeb'",
    ];
    let month_index = haab_day / 20;
    (day, month_names[month_index as usize])
}

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

fn get_tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("Ajaw", "C:/users/phine/desktop/tzolk'in/glyphs/ajaw.png");
    glyphs.insert("Imix", "C:/users/phine/desktop/tzolk'in/glyphs/imix.png");
    glyphs.insert("Ik'", "C:/users/phine/desktop/tzolk'in/glyphs/ik'.png");
    glyphs.insert("Ak'b'al", "C:/users/phine/desktop/tzolk'in/glyphs/ak'b'al.png");
    glyphs.insert("K'an", "C:/users/phine/desktop/tzolk'in/glyphs/ka'n.png");
    glyphs.insert("Chikchan", "C:/users/phine/desktop/tzolk'in/glyphs/chikchan.png");
    glyphs.insert("Kimi", "C:/users/phine/desktop/tzolk'in/glyphs/kimi.png");
    glyphs.insert("Manik'", "C:/users/phine/desktop/tzolk'in/glyphs/manik'.png");
    glyphs.insert("Lamat", "C:/users/phine/desktop/tzolk'in/glyphs/lamat.png");
    glyphs.insert("Muluk", "C:/users/phine/desktop/tzolk'in/glyphs/muluk.png");
    glyphs.insert("Ok", "C:/users/phine/desktop/tzolk'in/glyphs/ok.png");
    glyphs.insert("Chuwen", "C:/users/phine/desktop/tzolk'in/glyphs/chuwen.png");
    glyphs.insert("Eb'", "C:/users/phine/desktop/tzolk'in/glyphs/eb'.png");
    glyphs.insert("B'en", "C:/users/phine/desktop/tzolk'in/glyphs/b'en.png");
    glyphs.insert("Ix", "C:/users/phine/desktop/tzolk'in/glyphs/ix.png");
    glyphs.insert("Men", "C:/users/phine/desktop/tzolk'in/glyphs/men.png");
    glyphs.insert("K'ib'", "C:/users/phine/desktop/tzolk'in/glyphs/k'ib'.png");
    glyphs.insert("Kab'an", "C:/users/phine/desktop/tzolk'in/glyphs/kab'an.png");
    glyphs.insert("Etz'nab'", "C:/users/phine/desktop/tzolk'in/glyphs/etz'nab'.png");
    glyphs.insert("Kawak", "C:/users/phine/desktop/tzolk'in/glyphs/kawak'.png");
    glyphs
}

fn load_image_as_texture(ctx: &Context, path: &str) -> Result<eframe::egui::TextureHandle, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    if width != 128 || height != 128 {
        return Err(format!(
            "Image dimensions do not match the expected size: got {}x{}, expected 128x128.",
            width, height
        ));
    }
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );
    Ok(ctx.load_texture("Tzolk'in Glyph", color_image, TextureOptions::default()))
}

fn ui_example(ui: &mut Ui, ctx: &Context) {
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    let jdn = gregorian_to_jdn(year, month, day);
    let days_since_creation = jdn - 584283;

    let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);
    let (tzolkin_number, tzolkin_name) = tzolkin_date(days_since_creation);
    let (haab_day, haab_month) = haab_date(days_since_creation);
    let moon = moon_phase(jdn);

    let glyphs = get_tzolkin_glyphs();
    ui.vertical(|ui| {
        ui.label("Mayan Date:");
        ui.label(format!("13 {} (Tzolk'in)", tzolkin_name));

        ui.label(format!("📅 Gregorian Date: {}-{:02}-{:02}", year, month, day));
        ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin));
        ui.label(format!("🌞 Tzolk'in Date: {} {}", tzolkin_number, tzolkin_name));
        ui.label(format!("🌙 Haab’ Date: {} {}", haab_day, haab_month));
        ui.label(format!("🌕 Moon Phase: {}", moon));

        if let Some(image_path) = glyphs.get(tzolkin_name) {
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

struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui_example(ui, ctx);
        });
    }
}

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
