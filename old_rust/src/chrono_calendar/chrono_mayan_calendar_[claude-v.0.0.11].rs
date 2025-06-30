use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use image::io::Reader as ImageReader;
use eframe::egui::{Context, ColorImage, TextureHandle, TextureOptions};

/// Manages loading and caching of Mayan calendar glyphs
#[derive(Debug)]
pub struct AssetManager {
    tzolkin_textures: HashMap<String, TextureHandle>,
    haab_textures: HashMap<String, TextureHandle>,
    base_path: PathBuf,
}

impl AssetManager {
    /// Creates a new AssetManager with the given base path for assets
    pub fn new(ctx: &Context, base_path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let base_path = base_path.as_ref().to_path_buf();
        let mut manager = Self {
            tzolkin_textures: HashMap::new(),
            haab_textures: HashMap::new(),
            base_path,
        };

        // Load Tzolk'in glyphs
        for name in Self::tzolkin_day_names() {
            if let Ok(texture) = manager.load_glyph(ctx, name, "tzolkin") {
                manager.tzolkin_textures.insert(name.to_string(), texture);
            }
        }

        // Load Haab' glyphs
        for name in Self::haab_month_names() {
            if let Ok(texture) = manager.load_glyph(ctx, name, "haab") {
                manager.haab_textures.insert(name.to_string(), texture);
            }
        }

        Ok(manager)
    }

    /// Loads a glyph image and converts it to a texture
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

    pub fn get_tzolkin_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.tzolkin_textures.get(name)
    }

    pub fn get_haab_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.haab_textures.get(name)
    }

    /// Returns the list of Tzolk'in day names in Yucatec Maya
    pub fn tzolkin_day_names() -> &'static [&'static str] {
        &[
            "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
            "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
            "Chuwen", "Eb'", "B'en", "Ix", "Men",
            "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
        ]
    }

    /// Returns the list of Tzolk'in day names in K'iche' Maya
    pub fn tzolkin_kiche_names() -> &'static [&'static str] {
        &[
            "Imox", "Iq'", "Aq'ab'al", "K'at", "Kan",
            "Kame", "Kej", "Q'anil", "Tojil", "Tz'i'",
            "B'atz'", "E", "Aj", "Ix", "Tz'ikin",
            "Ajmaq", "No'j", "Tijax", "Kawoq", "Ajpu"
        ]
    }

    /// Returns the list of Haab' month names in Yucatec Maya
    pub fn haab_month_names() -> &'static [&'static str] {
        &[
            "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
            "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
            "Kayab", "Kumk'u", "Wayeb'"
        ]
    }

    /// Returns the list of Haab' month names in K'iche' Maya
    pub fn haab_kiche_names() -> &'static [&'static str] {
        &[
            "Pop", "Wo'", "Sip", "Zotz'", "Tzek", "Xul", "Yaxkin", "Mol",
            "Chen", "Yax", "Zac", "Keh", "Mak", "Kank'in", "Muwan", "Pax",
            "Kayab", "Kumk'u", "Wayeb'"
        ]
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