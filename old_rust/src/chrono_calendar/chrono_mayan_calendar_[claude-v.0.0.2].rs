// First, declare all the modules:
mod calendar_core { /* calendar_core code here */ }
mod mayan_cycles { /* mayan_cycles code here */ }
mod astronomical { /* astronomical code here */ }
mod glyphs { /* glyphs code here */ }
mod historical { /* historical code here */ }


use chrono::{Datelike, NaiveDate, Utc, Timelike};
use eframe::egui::{self, ColorImage, Context, TextureOptions};
use eframe::{App, Frame};
use std::collections::HashMap;
use std::path::PathBuf;

// Core Calendar Calculations

// Calendar Core Module
mod calendar_core {
    /// Convert a Gregorian date to Julian Day Number (JDN)
    pub fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }

    /// Long Count calculation from days since creation
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
}

// Mayan Cycles Module
mod mayan_cycles {
    pub struct TzolkinDate {
        pub number: i32,
        pub yucatec_name: &'static str,
        pub kiche_name: &'static str,
    }

    pub struct HaabDate {
        pub day: i32,
        pub yucatec_month: &'static str,
        pub kiche_month: &'static str,
    }

    pub fn tzolkin_date(days: i32) -> TzolkinDate {
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

    pub fn haab_date(days: i32) -> HaabDate {
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
        
        HaabDate {
            day,
            yucatec_month: yucatec_months[month_index as usize],
            kiche_month: kiche_months[month_index as usize],
        }
    }

    pub fn year_bearer(jdn: i32) -> &'static str {
        let tzolkin_days = ["Ik'", "Manik'", "Eb'", "K'an"];
        let year_start_tzolkin_index = (((jdn + 348) % 260) % 4) as usize;
        tzolkin_days[year_start_tzolkin_index]
    }
}

// Astronomical Module
mod astronomical {
    use chrono::NaiveDate;
    
    pub fn moon_phase(jdn: i32) -> &'static str {
        let synodic_month = 29.530588;
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

    pub fn venus_phase(jdn: i32) -> &'static str {
        let venus_cycle = 584;
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

    pub fn next_solstice_or_equinox(year: i32, month: i32, day: i32) -> (&'static str, i32) {
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
        
        ("🌸 Spring Equinox", 365 - (today.ordinal() - 79) as i32)
    }

    pub fn next_eclipse(jdn: i32) -> &'static str {
        let saros_cycle = 6585;
        let days_since_last_eclipse = jdn % saros_cycle;

        if days_since_last_eclipse < 15 {
            "🌑 Lunar Eclipse Soon!"
        } else if days_since_last_eclipse < 30 {
            "🌞 Solar Eclipse Soon!"
        } else {
            "🌘 No Eclipse Imminent"
        }
    }
}

// Glyphs Module
mod glyphs {
    pub fn mayan_numeral(n: i32) -> char {
        match n {
            0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap(),
            _ => '❓',
        }
    }

    pub fn mayan_ascii_number(n: i32) -> String {
        let mut result = String::new();
        let bars = n / 5;
        let dots = n % 5;

        for _ in 0..bars {
            result.push_str("▬▬▬▬▬▬\n");
        }

        if dots > 0 {
            for _ in 0..dots {
                result.push('●');
            }
            result.push('\n');
        }

        if n == 0 {
            result.push_str("𝋠\n");
        }

        result
    }
}

// Historical Module
mod historical {
    use super::calendar_core::gregorian_to_jdn;

    pub fn historical_event(jdn: i32) -> Option<&'static str> {
        let events = [
            (-3113, 8, 11, "🌎 The Maya creation date (0.0.0.0.0)"),
            (292, 1, 1, "📜 Earliest Long Count Date Found"),
            (378, 1, 16, "⚔️ Teotihuacan Influence Over Tikal Begins"),
            (426, 1, 1, "🏛️ Dynasty of Copán Founded"),
            (562, 1, 1, "🛑 Tikal Defeated by Calakmul"),
            (682, 6, 3, "👑 King Jasaw Chan K'awiil I Crowned in Tikal"),
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
}

// Bring module contents into scope
use calendar_core::*;
use mayan_cycles::*;
use astronomical::*;
use glyphs::*;
use historical::*;

// Constants
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const GLYPH_SIZE: u32 = 128;

// Asset handling functions
fn get_asset_path(asset_type: &str, filename: &str) -> PathBuf {
    PathBuf::from("assets")
        .join(asset_type)
        .join("glyphs")
        .join(filename)
}

fn load_glyph_as_texture(ctx: &Context, path: &PathBuf) -> Result<egui::TextureHandle, String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to open image {}: {}", path.display(), e))?;
    
    let img = img.resize_exact(GLYPH_SIZE, GLYPH_SIZE, image::imageops::FilterType::Lanczos3);
    let img = img.to_rgba8();
    
    let color_image = ColorImage::from_rgba_unmultiplied(
        [GLYPH_SIZE as usize, GLYPH_SIZE as usize],
        &img.into_raw(),
    );
    
    Ok(ctx.load_texture(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("glyph"),
        color_image,
        TextureOptions::default()
    ))
}

// Main application struct and implementation
struct MayanCalendar {
    current_time: chrono::NaiveTime,
    tzolkin_textures: HashMap<String, egui::TextureHandle>,
    haab_textures: HashMap<String, egui::TextureHandle>,
}

impl MayanCalendar {
    fn new(ctx: &Context) -> Result<Self, String> {
        let mut tzolkin_textures = HashMap::new();
        let mut haab_textures = HashMap::new();
        
        // Load textures on startup
        for (name, _) in get_tzolkin_glyphs().iter() {
            let asset_path = get_asset_path("tzolkin", &format!("{}.png", name));
            tzolkin_textures.insert(
                name.to_string(),
                load_glyph_as_texture(ctx, &asset_path)?
            );
        }
        
        Ok(Self {
            current_time: chrono::Local::now().time(),
            tzolkin_textures,
            haab_textures,
        })
    }
    
    fn render(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            self.render_calendar_side(ui);
            ui.separator();
            self.render_clock_side(ui);
        });
    }
    
    fn render_calendar_side(&self, ui: &mut egui::Ui) {
        let now = chrono::Utc::now().date_naive();
        let year = now.year();
        let month = now.month() as i32;
        let day = now.day() as i32;

        let jdn = gregorian_to_jdn(year, month, day);
        let days_since_creation = jdn - 584283;

        let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);
        let tzolkin = tzolkin_date(days_since_creation);
        let haab = haab_date(days_since_creation);
        let moon = moon_phase(jdn);
        let venus = venus_phase
        let (solstice, days_until) = next_solstice_or_equinox(year, month, day);
        let eclipse = next_eclipse(jdn);
  
        ui.vertical(|ui| {
            ui.heading("Mayan Date:");
            ui.label(format!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day));
            ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin));
            
            // Long Count ASCII Art
            ui.label("📜 Long Count (ASCII):");
            ui.label(format!("Baktun:\n{}", mayan_ascii_number(baktun)));
            ui.label(format!("Katun:\n{}", mayan_ascii_number(katun)));
            ui.label(format!("Tun:\n{}", mayan_ascii_number(tun)));
            ui.label(format!("Uinal:\n{}", mayan_ascii_number(uinal)));
            ui.label(format!("Kin:\n{}", mayan_ascii_number(kin)));
  
            // Calendar info
            ui.label(format!("🌞 Tzolk'in Date: {} {}", tzolkin.number, tzolkin.yucatec_name));
            ui.label(format!("🌙 Haab' Date: {} {}", haab.day, haab.yucatec_month));
            ui.label(format!("🌞 Year Bearer: {}", year_bearer(jdn)));
            ui.label(format!("🌙 Moon Phase: {}", moon));
            ui.label(format!("✨ Venus Cycle: {}", venus));
            ui.label(format!("🌓 Next Solstice/Equinox: {} ({} days away)", solstice, days_until));
            ui.label(format!("🌘 Eclipse Prediction: {}", eclipse));
  
            // Historical events
            if let Some(event) = historical_event(jdn) {
                ui.label(format!("🏛️ Historical Event Today: {}", event));
            } else {
                ui.label("📜 No significant historical event today.");
            }
        });
    }
    
    fn render_clock_side(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Digital clock display
            let time = self.current_time;
            ui.heading(format!(
                "{}:{:02}:{:02}",
                time.hour(),
                time.minute(),
                time.second()
            ));
  
            // Show Tzolk'in and Haab' glyphs if available
            if let Some(texture) = self.tzolkin_textures.get("Ok") {
                ui.image(texture);
            }
            if let Some(texture) = self.haab_textures.get("Pax") {
                ui.image(texture);
            }
        });
    }
  }
  
  impl eframe::App for MayanCalendar {
      fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
          self.current_time = chrono::Local::now().time();
          
          egui::CentralPanel::default().show(ctx, |ui| {
              egui::ScrollArea::vertical().show(ui, |ui| {
                  self.render(ui, ctx);
              });
          });
          
          ctx.request_repaint_after(std::time::Duration::from_secs(1));
      }
  }
  
  // Main function
  fn main() -> Result<(), eframe::Error> {
      let options = eframe::NativeOptions {
          initial_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
          ..Default::default()
      };
  
      eframe::run_native(
          "Mayan Calendar Clock",
          options,
          Box::new(|cc| {
              cc.egui_ctx.set_fonts(configure_fonts());
              Box::new(MayanCalendar::new(&cc.egui_ctx).expect("Failed to initialize calendar"))
          }),
      )
  }