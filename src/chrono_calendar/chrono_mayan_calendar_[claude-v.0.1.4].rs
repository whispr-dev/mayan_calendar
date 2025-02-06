use chrono::{Datelike, Timelike, Local};  // Removed NaiveDate
use eframe::egui::{self, Context, ScrollArea, Ui, ColorImage, TextureHandle, TextureOptions};
use eframe::{App, Frame};
use std::collections::HashMap;
use std::error::Error;
use image::io::Reader as ImageReader;

// Enhanced AssetManager with proper error handling
#[derive(Debug)]
struct AssetManager {
    tzolkin_textures: HashMap<String, TextureHandle>,
    haab_textures: HashMap<String, TextureHandle>,
    base_path: std::path::PathBuf,
}

impl AssetManager {  // Changed from "mpl" to "impl"
    fn new(ctx: &Context, base_path: &str) -> Result<Self, Box<dyn Error>> {
        let base_path = std::path::PathBuf::from(base_path);
        let mut manager = Self {
            tzolkin_textures: HashMap::new(),
            haab_textures: HashMap::new(),
            base_path,
        };

        fn load_glyph(&self, ctx: &Context, name: &str, category: &str) -> Result<TextureHandle, Box<dyn Error>> {
            let path = self.base_path.join(format!("assets/{}/{}.png", category, name));
            let img = ImageReader::open(path)?.decode()?;
            let size = [img.width() as _, img.height() as _];
            let pixels = img.to_rgba8().to_vec();
            
            Ok(ctx.load_texture(
                name,
                ColorImage::from_rgba_unmultiplied(size, &pixels),
                TextureOptions::default(),
            ))
        }

        impl std::fmt::Debug for AssetManager {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("AssetManager")
                    .field("base_path", &self.base_path)
                    .field("tzolkin_textures_count", &self.tzolkin_textures.len())
                    .field("haab_textures_count", &self.haab_textures.len())
                    .finish()
            }
        }        

        fn get_tzolkin_texture(&self, name: &str) -> Option<&TextureHandle> {
            self.tzolkin_textures.get(name)
        }
    
        fn get_haab_texture(&self, name: &str) -> Option<&TextureHandle> {
            self.haab_textures.get(name)
        }
    }

            fn tzolkin_day_names() -> [&'static str; 20] {
                [
                    "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
                    "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
                    "Chuwen", "Eb'", "B'en", "Ix", "Men",
                    "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
                ]
            }
        
            fn tzolkin_kiche_names() -> [&'static str; 20] {
                [
                    "B'atz'", "E", "Aj", "I'x", "Tz'ikin",
                    "Ajmaq", "No'j", "Tijax", "Kawoq", "Ajpu",
                    "Imox", "Iq'", "Aq'ab'al", "K'at", "Kan",
                    "Kame", "Kej", "Q'anil", "Toj", "Tz'i'"
                ]
            }
        
            fn haab_month_names() -> [&'static str; 19] {
                [
                    "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
                    "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
                    "Kayab", "Kumk'u", "Wayeb'"
                ]
            }
        
            fn haab_kiche_names() -> [&'static str; 19] {
                [
                    "Pop", "Wo", "Sip", "Sotz'", "Sek", "Xul", "Yaxk'in", "Mol",
                    "Ch'en", "Yax", "Sak", "Kej", "Mak", "K'ank'in", "Mwan", "Pax",
                    "Kayab'", "Kumk'u", "Wayeb'"
                ]
            }
        }

        // Load Tzolk'in glyphs
        for name in [
            "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
            "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
            "Chuwen", "Eb'", "B'en", "Ix", "Men",
            "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
        ] {
            if let Ok(texture) = manager.load_glyph(ctx, name, "tzolkin") {
                manager.tzolkin_textures.insert(name.to_string(), texture);
            }
        }

        // Load Haab' glyphs
        for name in [
            "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
            "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
            "Kayab", "Kumk'u", "Wayeb'"
    ] {
            if let Ok(texture) = manager.load_glyph(ctx, name, "haab") {
                manager.haab_textures.insert(name.to_string(), texture);
            }
        }

        Ok(manager)
    }


/// Represents a date in the Tzolk'in calendar
#[derive(Debug)]
pub struct TzolkinDate {
    pub number: i32,
    pub yucatec_name: &'static str,
    pub kiche_name: &'static str,
}

impl TzolkinDate {
    /// Creates a new Tzolk'in date from the number of days since the epoch
    pub fn new(days: i32) -> Self {
        let number = (((days + 3) % 13 + 13) % 13) + 1;
        let index = (((days + 19) % 20 + 20) % 20) as usize;
        
        Self {
            number,
            yucatec_name: AssetManager::tzolkin_day_names()[index],
            kiche_name: AssetManager::tzolkin_kiche_names()[index],
        }
    }
}

/// Represents a date in the Haab' calendar
#[derive(Debug)]
pub struct HaabDate {
    pub day: i32,
    pub yucatec_month: &'static str,
    pub kiche_month: &'static str,
}

impl HaabDate {
    /// Creates a new Haab' date from the number of days since the epoch
    pub fn new(days: i32) -> Self {
        let haab_day = ((days + 348) % 365 + 365) % 365;
        let month_index = haab_day / 20;
        let day = haab_day % 20;
        
        Self {
            day,
            yucatec_month: AssetManager::haab_month_names()[month_index as usize],
            kiche_month: AssetManager::haab_kiche_names()[month_index as usize],
        }
    }
}

/// Convert a Gregorian date to Julian Day Number (JDN)
pub fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Calculate Long Count components from days since creation
pub fn long_count(days: i32) -> (i32, i32, i32, i32, i32) {
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
pub fn mayan_numeral(n: i32) -> char {
    match n {
        0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap_or('?'),
        _ => '❓',
    }
}

/// Generate an ASCII-art representation of a Mayan number
pub fn mayan_ascii_number(n: i32) -> String {
    let mut result = String::new();

    // Handle zero specially
    if n == 0 {
        return "𝋠\n".to_string();
    }

    // Add bars (one per line)
    let bars = n / 5;
    for _ in 0..bars {
        result.push_str("▬▬▬▬▬▬\n");
    }

    // Add dots on a single line
    let dots = n % 5;
    if dots > 0 {
        for _ in 0..dots {
            result.push('●');
        }
        result.push('\n');
    }

    result
}

/// Calculate astronomical information for a given date
pub mod astronomy {
    /// Calculate the moon phase for a given Julian Day Number
    pub fn moon_phase(jdn: i32) -> &'static str {
        let synodic_month = 29.530588;
        let moon_age = (jdn as f64 % synodic_month) / synodic_month;

        match moon_age {
            x if x < 0.1 => "🌑 New Moon",
            x if x < 0.25 => "🌓 First Quarter",
            x if x < 0.5 => "🌕 Full Moon",
            x if x < 0.75 => "🌗 Last Quarter",
            _ => "🌑 New Moon",
        }
    }

    /// Calculate the Venus phase for a given Julian Day Number
    pub fn venus_phase(jdn: i32) -> &'static str {
        let venus_cycle = 584;
        let phase = jdn % venus_cycle;

        match phase {
            x if x < 50 => "🌟 Morning Star (Heliacal Rise)",
            x if x < 215 => "☀️ Superior Conjunction (Invisible)",
            x if x < 265 => "⭐ Evening Star (Heliacal Set)",
            _ => "🌑 Inferior Conjunction (Between Earth & Sun)",
        }
    }

    /// Predict eclipses based on the Saros cycle
    pub fn predict_eclipse(jdn: i32) -> &'static str {
        let saros_cycle = 6585;
        let days_since_last_eclipse = jdn % saros_cycle;

        match days_since_last_eclipse {
            x if x < 15 => "🌑 Lunar Eclipse Soon!",
            x if x < 30 => "🌞 Solar Eclipse Soon!",
            _ => "🌘 No Eclipse Imminent",
        }
    }
}

#[derive(Debug)]
struct CalendarState {
    long_count: (i32, i32, i32, i32, i32),
    tzolkin: TzolkinDate,
    haab: HaabDate,
    moon_phase: String,
    venus_phase: String,
    solstice_info: (String, i32),
    eclipse_prediction: String,
    historical_event: Option<String>,
}

impl CalendarState {
    fn new() -> Self {
        let today = Local::now().naive_local().date();
        let jdn = gregorian_to_jdn(
            today.year(),
            today.month() as i32,
            today.day() as i32
        );
        
        // Calculate days since Mayan epoch (August 11, 3114 BCE)
        let maya_epoch_jdn = gregorian_to_jdn(-3113, 8, 11);
        let days_since_epoch = jdn - maya_epoch_jdn;
        
        Self {
            long_count: long_count(days_since_epoch),
            tzolkin: TzolkinDate::new(days_since_epoch),
            haab: HaabDate::new(days_since_epoch),
            moon_phase: astronomy::moon_phase(jdn).to_string(),
            venus_phase: astronomy::venus_phase(jdn).to_string(),
            solstice_info: ("Winter Solstice".to_string(), 21), // This should be calculated properly
            eclipse_prediction: astronomy::predict_eclipse(jdn).to_string(),
            historical_event: None,
        }
    }
}

struct MayanCalendar {
    state: CalendarState,
    asset_manager: AssetManager,
    current_time: chrono::NaiveTime,
}

impl MayanCalendar {
    fn new(ctx: &Context) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            state: CalendarState::new(),
            asset_manager: AssetManager::new(ctx, "C:/users/phine/documents/github/mayan_calendar/src")?,
            current_time: Local::now().time(),
        })
    }

    fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.render_calendar_side(ui);
            ui.separator();
            self.render_clock_side(ui);
        });
    }

    fn render_calendar_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Your existing calendar rendering code...
            ui.heading("Mayan Calendar");
            ui.add_space(8.0);
            
            let (baktun, katun, tun, uinal, kin) = self.state.long_count;
            ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", 
                baktun, katun, tun, uinal, kin));
            
            ui.label(format!("🌞 Tzolk'in: {} {}", 
                self.state.tzolkin.number, self.state.tzolkin.yucatec_name));
            ui.label(format!("🌙 Haab': {} {}", 
                self.state.haab.day, self.state.haab.yucatec_month));
            
            ui.add_space(8.0);
            ui.label(&self.state.moon_phase);
            ui.label(&self.state.venus_phase);
            ui.label(format!("🌓 Next {}: {} days", 
                self.state.solstice_info.0, self.state.solstice_info.1));
            ui.label(&self.state.eclipse_prediction);
            
            if let Some(event) = &self.state.historical_event {
                ui.label(format!("🏛️ Historical Event: {}", event));
            }
        });
    }

    fn render_clock_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
            
            // Render glyphs in a horizontal layout
            ui.horizontal(|ui| {
                if let Some(texture) = self.asset_manager.get_tzolkin_texture(
                    self.state.tzolkin.yucatec_name
                ) {
                    ui.image(texture);
                }
                
                ui.add_space(16.0);
                
                if let Some(texture) = self.asset_manager.get_haab_texture(
                    self.state.haab.yucatec_month
                ) {
                    ui.image(texture);
                }
            });
        });
    }
}

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.current_time = Local::now().time();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render(ui);
            });
        });
        
        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            let app = MayanCalendar::new(&cc.egui_ctx).unwrap();
            Ok(Box::new(app) as Box<dyn App>)  // Wrap in Ok()
        }),
    )?;
    
    Ok(())
}
