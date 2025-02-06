use chrono::{Datelike, NaiveDate, Timelike, Local};
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

impl AssetManager {
    fn new(ctx: &Context, base_path: &str) -> Result<Self, Box<dyn Error>> {
        let base_path = std::path::PathBuf::from(base_path);
        let mut manager = Self {
            tzolkin_textures: HashMap::new(),
            haab_textures: HashMap::new(),
            base_path,
        };

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

    fn load_glyph(&self, ctx: &Context, name: &str, glyph_type: &str) -> Result<TextureHandle, Box<dyn Error>> {
        let file_name = format!("{}.png", name.to_lowercase().replace("'", ""));
        let path = self.base_path
            .join(glyph_type)
            .join("glyphs")
            .join(file_name);

        let img = ImageReader::open(&path)?.decode()?;
        let img = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
        let img = img.to_rgba8();
        
        let color_image = ColorImage::from_rgba_unmultiplied(
            [128, 128],
            &img.into_raw(),
        );

        Ok(ctx.load_texture(
            &format!("{}-{}", glyph_type, name),
            color_image,
            TextureOptions::default(),
        ))
    }

    fn get_tzolkin_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.tzolkin_textures.get(name)
    }

    fn get_haab_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.haab_textures.get(name)
    }
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
            Box::new(app) as Box<dyn App>
        }),
    )?;

    Ok(())
}

impl MayanCalendar {
    fn render_calendar_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Mayan Calendar");
            ui.add_space(8.0);
            
            // Long Count
            let (baktun, katun, tun, uinal, kin) = self.state.long_count;
            ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", 
                baktun, katun, tun, uinal, kin));
            
            // Long Count ASCII Art
            ui.collapsing("Long Count ASCII", |ui| {
                ui.monospace(format!("Baktun:\n{}", mayan_ascii_number(baktun)));
                ui.monospace(format!("Katun:\n{}", mayan_ascii_number(katun)));
                ui.monospace(format!("Tun:\n{}", mayan_ascii_number(tun)));
                ui.monospace(format!("Uinal:\n{}", mayan_ascii_number(uinal)));
                ui.monospace(format!("Kin:\n{}", mayan_ascii_number(kin)));
            });
            
            // Calendar Rounds
            ui.label(format!("🌞 Tzolk'in: {} {} (K'iche': {})", 
                self.state.tzolkin.number, 
                self.state.tzolkin.yucatec_name,
                self.state.tzolkin.kiche_name));
                
            ui.label(format!("🌙 Haab': {} {} (K'iche': {})", 
                self.state.haab.day,
                self.state.haab.yucatec_month,
                self.state.haab.kiche_month));
            
            // Astronomical Information
            ui.add_space(8.0);
            ui.label(&self.state.moon_phase);
            ui.label(&self.state.venus_phase);
            ui.label(format!("🌓 Next {}: {} days", 
                self.state.solstice_info.0,
                self.state.solstice_info.1));
            ui.label(&self.state.eclipse_prediction);
            
            // Historical Events
            if let Some(event) = &self.state.historical_event {
                ui.label(format!("🏛️ Historical Event: {}", event));
            }
        });
    }

    fn render_clock_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Digital Clock Display
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
            
            ui.add_space(16.0);
            
            // Glyph Display
            ui.horizontal(|ui| {
                // Tzolk'in Glyph
                if let Some(texture) = self.asset_manager.get_tzolkin_texture(
                    &self.state.tzolkin.yucatec_name
                ) {
                    ui.image(texture.id(), [128.0, 128.0]);
                }
                
                ui.add_space(16.0);
                
                // Haab' Glyph
                if let Some(texture) = self.asset_manager.get_haab_texture(
                    &self.state.haab.yucatec_month
                ) {
                    ui.image(texture.id(), [128.0, 128.0]);
                }
            });
        });
    }

    fn update_time(&mut self) {
        self.current_time = Local::now().time();
        
        // Update calendar state every day at midnight
        if self.current_time.hour() == 0 && 
           self.current_time.minute() == 0 && 
           self.current_time.second() == 0 {
            self.state = CalendarState::new();
        }
    }
}

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.update_time();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render(ui);
            });
        });
        
        // Request continuous updates for the clock
        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            // Configure custom fonts for Mayan numerals
            configure_fonts(&cc.egui_ctx);
            
            // Create and return the app
            let app = MayanCalendar::new(&cc.egui_ctx).unwrap();
            Box::new(app) as Box<dyn App>
        }),
    )?;

    Ok(())
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    
    // Add Mayan numeral font
    fonts.font_data.insert(
        "NotoSansMayanNumerals".to_string(),
        FontData::from_static(include_bytes!(
            "fonts/NotoSansMayanNumerals-Regular.ttf"
        )),
    );
    
    // Use for both proportional and monospace
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(family)
            .or_default()
            .push("NotoSansMayanNumerals".to_string());
    }
    
    ctx.set_fonts(fonts);
}
