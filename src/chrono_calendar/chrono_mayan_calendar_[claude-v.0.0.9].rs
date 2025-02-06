use chrono::{Datelike, NaiveDate, Timelike, Local};
use eframe::egui::{self, Context, ScrollArea, Ui, ColorImage, TextureHandle, TextureOptions};
use eframe::{App, Frame};
use std::collections::HashMap;
use std::error::Error;
use image::io::Reader as ImageReader;

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

#[derive(Debug)]
pub struct TzolkinDate {
    pub number: i32,
    pub yucatec_name: &'static str,
    pub kiche_name: &'static str,
}

impl TzolkinDate {
    fn new(days: i32) -> Self {
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
        
        Self {
            number,
            yucatec_name: yucatec_names[index],
            kiche_name: kiche_names[index],
        }
    }
}

#[derive(Debug)]
pub struct HaabDate {
    pub day: i32,
    pub yucatec_month: &'static str,
    pub kiche_month: &'static str,
}

impl HaabDate {
    fn new(days: i32) -> Self {
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
        
        Self {
            day,
            yucatec_month: yucatec_months[month_index as usize],
            kiche_month: kiche_months[month_index as usize],
        }
    }
}

#[derive(Debug)]
struct CalendarState {
    jdn: i32,
    days_since_creation: i32,
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
        let now = Local::now().date_naive();
        let year = now.year();
        let month = now.month() as i32;
        let day = now.day() as i32;
        
        let jdn = Self::gregorian_to_jdn(year, month, day);
        let days_since_creation = jdn - 584283;
        
        Self {
            jdn,
            days_since_creation,
            long_count: Self::calculate_long_count(days_since_creation),
            tzolkin: TzolkinDate::new(days_since_creation),
            haab: HaabDate::new(days_since_creation),
            moon_phase: Self::calculate_moon_phase(jdn),
            venus_phase: Self::calculate_venus_phase(jdn),
            solstice_info: Self::calculate_next_solstice(year, month, day),
            eclipse_prediction: Self::predict_eclipse(jdn),
            historical_event: Self::find_historical_event(jdn),
        }
    }

    fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }

    fn calculate_long_count(days: i32) -> (i32, i32, i32, i32, i32) {
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

    fn calculate_moon_phase(jdn: i32) -> String {
        let synodic_month = 29.530588;
        let moon_age = (jdn as f64 % synodic_month) / synodic_month;

        String::from(match moon_age {
            x if x < 0.1 => "🌑 New Moon",
            x if x < 0.25 => "🌓 First Quarter",
            x if x < 0.5 => "🌕 Full Moon",
            x if x < 0.75 => "🌗 Last Quarter",
            _ => "🌑 New Moon",
        })
    }

    fn calculate_venus_phase(jdn: i32) -> String {
        let venus_cycle = 584;
        let phase = jdn % venus_cycle;

        String::from(match phase {
            x if x < 50 => "🌟 Morning Star (Heliacal Rise)",
            x if x < 215 => "☀️ Superior Conjunction (Invisible)",
            x if x < 265 => "⭐ Evening Star (Heliacal Set)",
            _ => "🌑 Inferior Conjunction",
        })
    }

    fn calculate_next_solstice(year: i32, month: i32, day: i32) -> (String, i32) {
        let today = NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();
        let events = [
            ("🌸 Spring Equinox", NaiveDate::from_ymd_opt(year, 3, 20).unwrap()),
            ("☀️ Summer Solstice", NaiveDate::from_ymd_opt(year, 6, 21).unwrap()),
            ("🍂 Autumn Equinox", NaiveDate::from_ymd_opt(year, 9, 22).unwrap()),
            ("❄️ Winter Solstice", NaiveDate::from_ymd_opt(year, 12, 21).unwrap()),
        ];

        for (name, date) in events.iter() {
            if *date >= today {
                return (name.to_string(), (*date - today).num_days() as i32);
            }
        }
        
        ("🌸 Spring Equinox".to_string(), 365 - (today.ordinal() - 79) as i32)
    }

    fn predict_eclipse(jdn: i32) -> String {
        let saros_cycle = 6585;
        let days_since_last_eclipse = jdn % saros_cycle;

        String::from(match days_since_last_eclipse {
            x if x < 15 => "🌑 Lunar Eclipse Soon!",
            x if x < 30 => "🌞 Solar Eclipse Soon!",
            _ => "🌘 No Eclipse Imminent",
        })
    }

    fn find_historical_event(jdn: i32) -> Option<String> {
        let events = [
            (-3113, 8, 11, "🌎 The Maya creation date (0.0.0.0.0)"),
            (292, 1, 1, "📜 Earliest Long Count Date Found"),
            (378, 1, 16, "⚔️ Teotihuacan Influence Over Tikal Begins"),
            (426, 1, 1, "🏛️ Dynasty of Copán Founded"),
            (562, 1, 1, "🛑 Tikal Defeated by Calakmul"),
            (682, 6, 3, "👑 King Jasaw Chan K'awiil I Crowned in Tikal"),
        ];

        for (year, month, day, desc) in events.iter() {
            let event_jdn = Self::gregorian_to_jdn(*year, *month, *day);
            if jdn == event_jdn {
                return Some(desc.to_string());
            }
        }
        None
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
            ui.heading("Mayan Calendar");
            ui.add_space(8.0);
            
            let (baktun, katun, tun, uinal, kin) = self.state.long_count;
            ui.label(format!("🔢 Long Count: {}.{}.