mod config;
use config::Config;
mod date_utils;
use crate::date_utils::{gregorian_to_jdn, tzolkin_date, haab_date, TzolkinDate, HaabDate};
use chrono::{
    Local, 
    NaiveDate, 
    NaiveDateTime, 
    Datelike,  // Add this for year(), month(), day(), ordinal() methods
    Timelike,  // For time-related methods
    Utc
};
use eframe::egui::{ColorImage, Context, TextureOptions, Ui};
use eframe::{App, Frame};
use std::collections::HashMap;
use eframe::egui;
use std::time::Instant;
use lazy_static::lazy_static;
use std::sync::Arc;
use parking_lot::RwLock;
use lru::LruCache;
use std::num::NonZeroUsize;
use rayon::prelude::*;
use memmap2::MmapOptions;
use std::fs::File;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use std::sync::atomic::{AtomicU64, Ordering};

// Performance metrics tracking
pub struct Metrics {
    calculation_time: AtomicU64,
    glyph_load_time: AtomicU64,
    render_time: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            calculation_time: AtomicU64::new(0),
            glyph_load_time: AtomicU64::new(0),
            render_time: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    pub fn record_calculation(&self, duration: std::time::Duration) {
        self.calculation_time.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn report(&self) -> String {
        format!(
            "Performance Metrics:\n\
             Calculation Time: {}µs\n\
             Cache Hits: {}\n\
             Cache Misses: {}\n\
             Cache Hit Rate: {:.2}%",
            self.calculation_time.load(Ordering::Relaxed),
            self.cache_hits.load(Ordering::Relaxed),
            self.cache_misses.load(Ordering::Relaxed),
            self.cache_hit_rate() * 100.0
        )
    }

    fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
}

// First, let's define our calendar constants. Using lazy_static allows us to 
// initialize complex static values at runtime while still maintaining efficiency
lazy_static! {
    // Tzolk'in day names with their K'iche' equivalents
    static ref TZOLKIN_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Imix", "B'atz'");
        m.insert("Ik'", "E'");
        m.insert("Ak'b'al", "Aj");
        m.insert("K'an", "I'x");
        m.insert("Chikchan", "Tz'ikin");
        m.insert("Kimi", "Ajmaq");
        m.insert("Manik'", "No'j");
        m.insert("Lamat", "Tijax");
        m.insert("Muluk", "Kawoq");
        m.insert("Ok", "Ajpu");
        m.insert("Chuwen", "Imox");
        m.insert("Eb'", "Iq'");
        m.insert("B'en", "Aq'ab'al");
        m.insert("Ix", "K'at");
        m.insert("Men", "Kan");
        m.insert("Kib'", "Kame");
        m.insert("Kab'an", "Kej");
        m.insert("Etz'nab'", "Q'anil");
        m.insert("Kawak", "Toj");
        m.insert("Ajaw", "Tz'i'");
        m
    };

    // Haab' month names with their K'iche' equivalents
    static ref HAAB_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Pop", "Nab'e Mam");
        m.insert("Wo'", "U Kab' Mam");
        m.insert("Sip", "Nab'e Pach");
        m.insert("Sotz'", "U Kab' Pach");
        m.insert("Sek", "Nab'e Mam");
        m.insert("Xul", "U Kab' Mam");
        m.insert("Yaxk'in", "Nab'e Toj");
        m.insert("Mol", "U Kab' Toj");
        m.insert("Ch'en", "Nab'e K'ij");
        m.insert("Yax", "U Kab' K'ij");
        m.insert("Sak'", "Nab'e Sak'ow");
        m.insert("Keh", "U Kab' Sak'ow");
        m.insert("Mak", "Nab'e Tz'ib'");
        m.insert("K'ank'in", "U Kab' Tz'ib'");
        m.insert("Muwan", "Nab'e Winaq");
        m.insert("Pax", "U Kab' Winaq");
        m.insert("K'ayab", "Nab'e Saq");
        m.insert("Kumk'u", "U Kab' Saq");
        m.insert("Wayeb'", "Wayeb'");
        m
    };


    // Historical events organized by Julian Day Number for efficient lookup
    static ref HISTORICAL_EVENTS: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        // Converting historical dates to JDN for direct lookup
        m.insert(584283, "🌎 The Maya creation date (0.0.0.0.0)");
        m.insert(1710534, "📜 Earliest Long Count Date Found");
        m.insert(1729974, "⚔️ Teotihuacan Influence Over Tikal Begins");
        m.insert(1747528, "🏛️ Dynasty of Copán Founded");
        m.insert(1787293, "🛑 Tikal Defeated by Calakmul");
        m.insert(1830475, "👑 King Jasaw Chan K'awiil I Crowned in Tikal");
        m.insert(1854923, "🏛️ Uxmal Emerges as a Major Power");
        m.insert(1898765, "🏛️ Tikal Abandoned");
        m.insert(1943872, "🏰 Toltec-Maya Rule in Chichen Itzá Begins");
        m.insert(2052647, "🔺 Decline of Chichen Itzá");
        m.insert(2160983, "⚔️ Spanish Make First Contact with the Maya");
        m.insert(2214876, "🏹 Spanish Conquer the Last Maya City, Tayasal");
        m
    };

    // Astronomical constants
    static ref ASTRONOMICAL_CYCLES: HashMap<&'static str, f64> = {
        let mut m = HashMap::new();
        m.insert("synodic_month", 29.530588); // Lunar cycle
        m.insert("venus_synodic", 583.92);    // Venus synodic period
        m.insert("solar_year", 365.242189);   // Tropical year
        m.insert("eclipse_year", 346.62);     // Eclipse year
        m
    };
}

//// Convert a number (0-19) to a Mayan numeral Unicode character
fn mayan_numeral(n: i32) -> char {
    match n {
        0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap(),
        _ => '❓', // If out of range, return a placeholder
    }
}

f// First, let's create a more efficient Long Count calculation system
// We'll use const functions where possible for compile-time evaluation
const BAKTUN_DAYS: i32 = 144_000;  // 20 * 18 * 20 * 20
const KATUN_DAYS: i32 = 7_200;     // 20 * 18 * 20
const TUN_DAYS: i32 = 360;         // 20 * 18
const UINAL_DAYS: i32 = 20;        // 20

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct LongCount {
    baktun: i32,
    katun: i32,
    tun: i32,
    uinal: i32,
    kin: i32,
}

impl LongCount {
    // Optimized conversion from days to Long Count
    pub fn from_days(days: i32) -> Self {
        // Use integer division and remainder for maximum efficiency
        let baktun = days / BAKTUN_DAYS;
        let rem1 = days % BAKTUN_DAYS;
        
        let katun = rem1 / KATUN_DAYS;
        let rem2 = rem1 % KATUN_DAYS;
        
        let tun = rem2 / TUN_DAYS;
        let rem3 = rem2 % TUN_DAYS;
        
        let uinal = rem3 / UINAL_DAYS;
        let kin = rem3 % UINAL_DAYS;

        Self { baktun, katun, tun, uinal, kin }
    }

    // Convert Long Count back to total days
    pub fn to_days(&self) -> i32 {
        self.baktun * BAKTUN_DAYS +
        self.katun * KATUN_DAYS +
        self.tun * TUN_DAYS +
        self.uinal * UINAL_DAYS +
        self.kin
    }
}

// Parallel calendar calculations
pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { metrics, cache }
    }

    // Calculate multiple dates in parallel
    pub fn calculate_date_range(&self, start_days: i32, count: i32) -> Vec<CalendarData> {
        let start = Instant::now();

        let results: Vec<CalendarData> = (0..count)
            .into_par_iter()
            .map(|offset| {
                let days = start_days + offset;
                self.calculate_single_date(days)
            })
            .collect();

        let duration = start.elapsed();
        self.metrics.record_calculation(duration);
        
        info!(
            target: "calendar_calculation",
            "Calculated {} dates in {}µs",
            count,
            duration.as_micros()
        );

        results
    }

    fn calculate_single_date(&self, days: i32) -> CalendarData {
        // Check cache first
        let cache_check_start = Instant::now();
        {
            let cache = self.cache.read();
            if let Some(data) = cache.get_calendar_data(days) {
                self.metrics.record_cache_hit();
                return data;
            }
        }
        self.metrics.record_cache_miss();

        // Calculate if not in cache
        let calc_start = Instant::now();
        let long_count = LongCount::from_days(days);
        let tzolkin = tzolkin_date(days);
        let haab = haab_date(days);
        
        let data = CalendarData::new_from_components(
            long_count,
            tzolkin,
            haab,
            days
        );

        // Cache the result
        let mut cache = self.cache.write();
        cache.put_calendar_data(days, data.clone());

        let duration = calc_start.elapsed();
        info!(
            target: "calendar_calculation",
            "Single date calculation took {}µs",
            duration.as_micros()
        );

        data
    }
}

// Updated Tzolk'in date calculation using precomputed names
fn tzolkin_date(days: i32) -> TzolkinDate {
    let number = ((days + 4) % 13) + 1;
    let position = ((days + 19) % 20) as usize;
    
    // Get the Yucatec name (we'll store these in order)
    let yucatec_names = [
        "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
        "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
        "Chuwen", "Eb'", "B'en", "Ix", "Men",
        "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
    ];
    
    let yucatec_name = yucatec_names[position];
    let kiche_name = TZOLKIN_NAMES.get(yucatec_name).copied().unwrap_or("Unknown");

    TzolkinDate {
        number,
        yucatec_name,
        kiche_name,
    }
}

// Updated Haab' date calculation using precomputed names
fn haab_date(days: i32) -> HaabDate {
    let year_position = days % 365;
    let day = year_position % 20;
    
    // Calculate month position (0-18)
    let month_position = (year_position / 20) as usize;
    
    // Get the Yucatec month name (we'll store these in order)
    let yucatec_months = [
        "Pop", "Wo'", "Sip", "Sotz'", "Sek",
        "Xul", "Yaxk'in", "Mol", "Ch'en", "Yax",
        "Sak'", "Keh", "Mak", "K'ank'in", "Muwan",
        "Pax", "K'ayab", "Kumk'u", "Wayeb'"
    ];
    
    let yucatec_month = yucatec_months[month_position];
    let kiche_month = HAAB_NAMES.get(yucatec_month).copied().unwrap_or("Unknown");

    HaabDate {
        day,
        yucatec_month,
        kiche_month,
    }
}

// Updated moon phase calculation using precomputed constants
fn moon_phase(jdn: i32) -> &'static str {
    let synodic_month = ASTRONOMICAL_CYCLES.get("synodic_month").unwrap();
    let moon_age = (jdn as f64 % synodic_month) / synodic_month;

    match moon_age {
        x if x < 0.1 => "🌑 New Moon",
        x if x < 0.25 => "🌓 First Quarter",
        x if x < 0.5 => "🌕 Full Moon",
        x if x < 0.75 => "🌗 Last Quarter",
        _ => "🌑 New Moon"
    }
}

/// Generate an ASCII-art Mayan Long Count representation
fn mayan_ascii_number(n: i32) -> String {
    let mut result = String::new();

    // Calculate the number of bars and dots
    let bars = n / 5;
    let dots = n % 5;

    // Add bars (one per line)
    for _ in 0..bars {
        result.push_str("▬▬▬▬▬▬\n"); // Full-width bar
    }

    // Add dots (on a single line after bars)
    if dots > 0 {
        for _ in 0..dots {
            result.push('●'); // Add a dot
        }
        result.push('\n'); // Newline after dots
    }

    // Handle zero (special Mayan zero glyph)
    if n == 0 {
        result.push_str("𝋠\n"); // Mayan zero glyph fallback
    }

    result
}

// Updated historical event function using efficient HashMap lookup
fn historical_event(jdn: i32) -> Option<&'static str> {
    HISTORICAL_EVENTS.get(&jdn).copied()
}

struct TextureCache {
    tzolkin_textures: HashMap<String, eframe::egui::TextureHandle>,
    haab_textures: HashMap<String, eframe::egui::TextureHandle>,
}


//   #  !!!!!! ########################

fn get_tzolkin_glyphs(config: &Config) -> HashMap<&str, &str> {
    config.tzolkin_glyphs
        .iter()
        .map(|(name, path)| (name.as_str(), path.as_str()))
        .collect()
}

fn get_haab_glyphs(config: &Config) -> HashMap<&str, &str> {
    config.haab_glyphs
        .iter()
        .map(|(name, path)| (name.as_str(), path.as_str()))
        .collect()
}

//  #  !!!!!! ##########################


/// Calculate Year Bearer (Patron Tzolk’in Day of Haab’ New Year)
fn year_bearer(jdn: i32) -> &'static str {
  let tzolkin_days = ["Ik'", "Manik'", "Eb'", "K’an"];
  let year_start_tzolkin_index = (((jdn + 348) % 260) % 4) as usize;
  tzolkin_days[year_start_tzolkin_index]
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

// Updated Venus phase calculation using precomputed constants
fn venus_phase(jdn: i32) -> &'static str {
    let venus_cycle = ASTRONOMICAL_CYCLES.get("venus_synodic").unwrap();
    let phase = (jdn as f64 % venus_cycle) as f64;

    match phase {
        x if x < 50.0 => "🌟 Morning Star (Heliacal Rise)",
        x if x < 215.0 => "☀️ Superior Conjunction (Invisible)",
        x if x < 265.0 => "⭐ Evening Star (Heliacal Set)",
        _ => "🌑 Inferior Conjunction (Between Earth & Sun)"
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
  ("🌸 Spring Equinox", 365 - (today.month() as i32 * 31 - 79) as i32)
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

// Enhanced GlyphRenderer with memory-mapped file access
pub struct GlyphRenderer {
    cache: Arc<RwLock<TextureCache>>,
    config: Config,
    metrics: Arc<Metrics>,
}

impl GlyphRenderer {
    pub fn new(ctx: &Context, config: Config) -> Self {
        // Initialize tracing subscriber for logging
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env()
                .add_directive(Level::INFO.into()))
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .compact()
            .init();

        Self {
            cache: Arc::new(RwLock::new(TextureCache {
                tzolkin_textures: HashMap::new(),
                haab_textures: HashMap::new(),
            })),
            config,
            metrics: Arc::new(Metrics::new()),
        }
    }

    // Memory-mapped file loading for glyphs
    fn load_glyph_mmap(&self, path: &str) -> Result<Vec<u8>, GlyphError> {
        let start = Instant::now();
        let file = File::open(path).map_err(|e| GlyphError::FileError(e))?;
        
        // Create a read-only memory map
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| GlyphError::MmapError(e))?
        };

        let duration = start.elapsed();
        info!(
            target: "glyph_loading",
            "Loaded glyph {} in {}µs using mmap",
            path,
            duration.as_micros()
        );

        Ok(mmap.to_vec())
    }

    // Parallel texture loading for multiple glyphs
    pub fn preload_glyphs(&self, ctx: &Context) -> Result<(), GlyphError> {
        let start = Instant::now();

        // Collect all glyph paths
        let mut paths = Vec::new();
        paths.extend(self.config.tzolkin_glyphs.values().cloned());
        paths.extend(self.config.haab_glyphs.values().cloned());

        // Load glyphs in parallel using rayon
        paths.par_iter().try_for_each(|path| {
            match self.load_glyph_mmap(path) {
                Ok(data) => {
                    // Process the loaded data and create texture
                    let img = image::load_from_memory(&data)
                        .map_err(GlyphError::ImageLoadError)?;
                    
                    let img = img.to_rgba8();
                    let (width, height) = img.dimensions();

                    if width != 128 || height != 128 {
                        return Err(GlyphError::InvalidDimensions(width, height));
                    }

                    // Create and cache the texture
                    let color_image = ColorImage::from_rgba_unmultiplied(
                        [width as usize, height as usize],
                        &img.into_raw(),
                    );

                    let texture = ctx.load_texture(
                        path,
                        color_image,
                        TextureOptions::default(),
                    );

                    // Update cache
                    let mut cache = self.cache.write();
                    if path.contains("tzolkin") {
                        cache.tzolkin_textures.insert(path.to_string(), texture);
                    } else {
                        cache.haab_textures.insert(path.to_string(), texture);
                    }

                    Ok(())
                }
                Err(e) => {
                    error!("Failed to load glyph {}: {}", path, e);
                    Err(e)
                }
            }
        })?;

        let duration = start.elapsed();
        info!(
            target: "glyph_loading",
            "Preloaded {} glyphs in {}ms",
            paths.len(),
            duration.as_millis()
        );

        Ok(())
    }
}

// Error handling for glyph loading
#[derive(Debug, thiserror::Error)]
pub enum GlyphError {
    #[error("Failed to load image: {0}")]
    ImageLoadError(#[from] image::ImageError),
    
    #[error("Invalid glyph dimensions: {0}x{1}, expected 128x128")]
    InvalidDimensions(u32, u32),
}

// UI Component functions for Mayan Calendar

// Renders the basic date information section
fn render_date_info(ui: &mut Ui, year: i32, month: i32, day: i32, long_count: (i32, i32, i32, i32, i32)) {
    let (baktun, katun, tun, uinal, kin) = long_count;
    
    ui.heading("Mayan Date:");
    ui.label(format!("📅 Gregorian Date: {}-{:02}-{:02}", year, month, day));
    ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin));
}

// Renders the Long Count section with different notation styles
fn render_long_count_displays(ui: &mut Ui, long_count: (i32, i32, i32, i32, i32)) {
    let (baktun, katun, tun, uinal, kin) = long_count;
    
    // Unicode Glyphs
    ui.label(format!(
        "📜 Long Count (Unicode): {}{}{}{}{}",
        mayan_numeral(baktun),
        mayan_numeral(katun),
        mayan_numeral(tun),
        mayan_numeral(uinal),
        mayan_numeral(kin)
    ));

    // ASCII Art representation
    ui.label("📜 Long Count (ASCII):");
    for (value, name) in [(baktun, "Baktun"), (katun, "Katun"), (tun, "Tun"), 
                         (uinal, "Uinal"), (kin, "Kin")] {
        ui.monospace(format!("{}:\n{}", name, mayan_ascii_number(value)));
    }
}

// Renders the Calendar Round section (Tzolk'in and Haab' dates)
fn render_calendar_round(ui: &mut Ui, tzolkin: &TzolkinDate, haab: &HaabDate) {
    ui.label(format!(
        "🌞 Tzolk'in Date: {} {} (K'iche': {})",
        tzolkin.number, tzolkin.yucatec_name, tzolkin.kiche_name
    ));
    ui.label(format!(
        "🌙 Haab' Date: {} {} (K'iche': {})",
        haab.day, haab.yucatec_month, haab.kiche_month
    ));
}

// Renders astronomical information
fn render_astronomical_info(ui: &mut Ui, bearer: &str, moon: &str, venus: &str) {
    ui.label(format!("🌞 Year Bearer: {}", bearer));
    ui.label(format!("🌕 Moon Phase: {}", moon));
    ui.label(format!("✨ Venus Cycle: {}", venus));
}

// Renders seasonal and celestial events
fn render_celestial_events(ui: &mut Ui, solstice: &str, days_until: i32, eclipse: &str) {
    ui.label(format!(
        "🌓 Next Solstice/Equinox: {} ({} days away)",
        solstice, days_until
    ));
    ui.label(format!("🌘 Eclipse Prediction: {}", eclipse));
}

// Renders historical event information
fn render_historical_info(ui: &mut Ui, historical: Option<&str>) {
    match historical {
        Some(event) => ui.label(format!("🏛️ Historical Event Today: {}", event)),
        None => ui.label("📜 No significant historical event today."),
    }
}

// Main UI rendering function (replaces ui_example)
fn render_mayan_calendar(ui: &mut Ui, ctx: &Context) {
    let now = Utc::now().date_naive();
    let year = now.year();
    let month = now.month() as i32;
    let day = now.day() as i32;

    let jdn = gregorian_to_jdn(year, month, day);
    let days_since_creation = jdn - 584283;

    // Calculate all calendar data
    let long_count = long_count(days_since_creation);
    let tzolkin = tzolkin_date(days_since_creation);
    let haab = haab_date(days_since_creation);
    let moon = moon_phase(jdn);
    let bearer = year_bearer(jdn);
    let venus = venus_phase(jdn);
    let (solstice, days_until) = next_solstice_or_equinox(year, month, day);
    let eclipse = next_eclipse(jdn);
    let historical = historical_event(jdn);

    // Render UI components
    ui.vertical(|ui| {
        render_date_info(ui, year, month, day, long_count);
        render_long_count_displays(ui, long_count);
        render_calendar_round(ui, &tzolkin, &haab);
        render_astronomical_info(ui, bearer, moon, venus);
        render_celestial_events(ui, solstice, days_until, eclipse);
        render_historical_info(ui, historical);

        // Render glyphs if calendar is available
        if let Ok(mut calendar) = MayanCalendar::new(ctx) {
            calendar.render_glyphs(ui, ctx, &tzolkin, &haab);
        }
    });
}

// Update the App implementation to use the new render_mayan_calendar function
impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let now = Local::now().naive_local();

        // Update time and calendar if needed
        if now.signed_duration_since(self.last_calendar_update).num_seconds() >= 1 {
            self.current_time = now.time();
            self.last_calendar_update = now;
            self.update_calendar_if_needed();
            ctx.request_repaint();
        }
        
        // Create the main window
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_clock_side(ui);
                render_mayan_calendar(ui, ctx);
            });
        });
    }
}

// Updated MayanCalendar struct with new optimizations
pub struct MayanCalendar {
    current_time: chrono::NaiveTime,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: CalendarCache,
    glyph_renderer: GlyphRenderer,
}

// Enhanced MayanCalendar implementation with parallel processing
impl MayanCalendar {
    pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        let metrics = Arc::new(Metrics::new());
        let cache = Arc::new(RwLock::new(CalendarCache::new(
            NonZeroUsize::new(100).unwrap()
        )));
        
        let calculator = ParallelCalendarCalculator::new(
            Arc::clone(&cache),
            Arc::clone(&metrics)
        );

        let now = Local::now();
        let config = Config::new();
        
        let mut calendar = Self {
            current_time: now.time(),
            calendar_data: CalendarData::new(now.naive_local()),
            last_calendar_update: now.naive_local(),
            cache,
            glyph_renderer: GlyphRenderer::new(ctx, config),
            calculator,
            metrics,
        };

        // Preload glyphs in parallel
        calendar.glyph_renderer.preload_glyphs(ctx)?;

        Ok(calendar)
    }

    // Generate performance report
    pub fn generate_performance_report(&self) -> String {
        self.metrics.report()
    }
}

struct CalendarData {
    // Long Count components
    long_count: (i32, i32, i32, i32, i32),  // (baktun, katun, tun, uinal, kin)
    
    // Calendar round components
    tzolkin: TzolkinDate,
    haab: HaabDate,
    
    // Astronomical information
    moon_phase: String,
    venus_phase: String,
    year_bearer: String,
    
    // Seasonal information
    next_solstice: (String, i32),
    
    // Eclipse prediction
    eclipse_status: String,
    
    // Historical information
    historical_event: Option<String>,
    
    // Base calendar information
    gregorian_date: NaiveDate,
    julian_day_number: i32,
    days_since_creation: i32,
}

impl CalendarData {
fn new(date: NaiveDateTime) -> Self {
    let naive_date = date.date();  // Convert to NaiveDate
    let year = naive_date.year();
    let month = naive_date.month() as i32;
    let day = naive_date.day() as i32;
        
        let jdn = gregorian_to_jdn(year, month, day);
        let days_since_creation = jdn - 584283;
        
        // Calculate Long Count
        let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);
        
        // Calculate calendar rounds
        let tzolkin = tzolkin_date(days_since_creation);
        let haab = haab_date(days_since_creation);
        
        // Calculate astronomical info
        let moon_phase = moon_phase(jdn).to_string();
        let venus_phase = venus_phase(jdn).to_string();
        let year_bearer = year_bearer(jdn).to_string();
        
        // Calculate seasonal info
        let (solstice_name, days_until) = next_solstice_or_equinox(year, month, day);
        
        // Get eclipse prediction
        let eclipse_status = next_eclipse(jdn).to_string();
        
        // Check for historical events
        let historical_event = historical_event(jdn).map(String::from);
        
        Self {
            long_count: (baktun, katun, tun, uinal, kin),
            tzolkin,
            haab,
            moon_phase,
            venus_phase,
            year_bearer,
            next_solstice: (solstice_name.to_string(), days_until),
            eclipse_status,
            historical_event,
            gregorian_date: date.date(),
            julian_day_number: jdn,
            days_since_creation,
        }
    }
}

i// Implementation of UI rendering using the new optimized components
impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let now = Local::now().naive_local();

        // Update time and calendar if needed
        if now.signed_duration_since(self.last_calendar_update).num_seconds() >= 1 {
            self.current_time = now.time();
            self.last_calendar_update = now;
            self.update_calendar_if_needed();
            ctx.request_repaint();
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_clock_side(ui);
                
                // Use cached values and optimized rendering
                let days_since_creation = self.calendar_data.days_since_creation;
                let long_count = self.cache.get_long_count(days_since_creation);
                
                // Render the calendar using the optimized components
                render_mayan_calendar(
                    ui,
                    ctx,
                    &long_count,
                    &self.calendar_data.tzolkin,
                    &self.calendar_data.haab,
                    &mut self.glyph_renderer,
                );
            });
        });
    }
}

    // Clock side rendering method
    fn render_clock_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
        });
    }

    // Update calendar if the date has changed
    fn update_calendar_if_needed(&mut self) {
        let now = Local::now().naive_local();
        if now.date() != self.last_calendar_update.date() {
            self.calendar_data = CalendarData::new(now);
            self.last_calendar_update = now;
        }
    }

    fn render_glyphs(&mut self, ui: &mut Ui, ctx: &Context, tzolkin: &TzolkinDate, haab: &HaabDate) {
       ui.horizontal(|ui| {
            let tzolkin_glyphs = get_tzolkin_glyphs(&self.config);
            if let Some(image_path) = tzolkin_glyphs.get(tzolkin.yucatec_name) {
                match load_glyph_texture(ctx, image_path, "tzolkin", &mut self.texture_cache) {
                    Ok(texture) => {
                        ui.image(&texture);
                    }
                    Err(err) => {
                        ui.label(format!("❌ Failed to load Tzolk'in glyph: {}", err));
                    }
                }
            }
    
            ui.add_space(16.0);
    
            let haab_glyphs = get_haab_glyphs(&self.config);
            if let Some(image_path) = haab_glyphs.get(haab.yucatec_month) {
                match load_glyph_texture(ctx, image_path, "haab", &mut self.texture_cache) {
                    Ok(texture) => {
                        ui.image(&texture);
                    }
                    Err(err) => {
                        ui.label(format!("❌ Failed to load Haab' glyph: {}", err));
                    }
                }
            });
        });
    }

// Implement the App trait
impl App for MayanCalendar {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
// Get the current time as a NaiveDateTime
let now = Local::now().naive_local();

// Check if a second has elapsed since the last update
if now.signed_duration_since(self.last_calendar_update).num_seconds() >= 1 {
    // Update the current time
    self.current_time = now.time();
    
    // Update the last update time
    self.last_calendar_update = now;
    
    // Update calendar if needed
    self.update_calendar_if_needed();
    
    // Request a repaint
    ctx.request_repaint();
}
        
        // Create the main window
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Clock display
                self.render_clock_side(ui);
                
                // Calendar display
                ui_example(ui, ctx);
            });
        });
    }
}

// Error handling for memory-mapped operations
#[derive(Debug, thiserror::Error)]
pub enum GlyphError {
    #[error("Failed to open file: {0}")]
    FileError(std::io::Error),
    
    #[error("Memory mapping failed: {0}")]
    MmapError(std::io::Error),
    
    #[error("Failed to load image: {0}")]
    ImageLoadError(#[from] image::ImageError),
    
    #[error("Invalid glyph dimensions: {0}x{1}, expected 128x128")]
    InvalidDimensions(u32, u32),
}

fn configure_fonts(ctx: &eframe::egui::Context) {
  use eframe::egui::{FontDefinitions, FontFamily, FontData};
  use std::sync::Arc;
  
  let mut fonts = FontDefinitions::default();
  
  let font_bytes = include_bytes!("fonts/NotoSansMayanNumerals-Regular.ttf");
  
  fonts.font_data.insert(
      "NotoSansMayanNumerals".to_string(),
      Arc::new(FontData::from_static(font_bytes))
  );

  // Rest of the configuration...
  fonts
      .families
      .entry(FontFamily::Proportional)
      .or_default()
      .insert(0, "NotoSansMayanNumerals".to_string());
  fonts
      .families
      .entry(FontFamily::Monospace)
      .or_default()
      .insert(0, "NotoSansMayanNumerals".to_string());

  ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            
            match MayanCalendar::new(&cc.egui_ctx) {
                Ok(app) => Ok(Box::new(app) as Box<dyn App>),
                Err(_) => {
                    let now = Local::now();
                    Ok(Box::new(MayanCalendar {
                        current_time: now.time(),
                        calendar_data: CalendarData::new(now.naive_local()),
                        last_calendar_update: now.naive_local(),
                        texture_cache: TextureCache {
                            tzolkin_textures: HashMap::new(),
                            haab_textures: HashMap::new(),
                        },
                    }) as Box<dyn App>)
                }
            }
        })
    )
}