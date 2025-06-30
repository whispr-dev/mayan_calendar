// First, let's organize our imports properly
use chrono::{Local, NaiveDate, NaiveDateTime, Datelike, Timelike, Utc};
use eframe::egui::{self, ColorImage, Context, TextureOptions, Ui};
use eframe::{App, Frame};
use lazy_static::lazy_static;
use lru::LruCache;
use memmap2::MmapOptions;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    num::NonZeroUsize,
    sync::{atomic::{AtomicU64, Ordering}, Arc},
    time::Instant,
};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};

// Module imports
mod config;
mod date_utils;
use config::Config;
use date_utils::{gregorian_to_jdn, tzolkin_date, haab_date, TzolkinDate, HaabDate};

// Constants for Long Count calculations
const BAKTUN_DAYS: i32 = 144_000;  // 20 * 18 * 20 * 20
const KATUN_DAYS: i32 = 7_200;     // 20 * 18 * 20
const TUN_DAYS: i32 = 360;         // 20 * 18
const UINAL_DAYS: i32 = 20;        // 20

// Calendar constants using lazy_static
lazy_static! {
    static ref TZOLKIN_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Imix", "B'atz'");
        // ... [rest of Tzolk'in names]
        m
    };

    static ref HAAB_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Pop", "Nab'e Mam");
        // ... [rest of Haab' names]
        m
    };

    static ref HISTORICAL_EVENTS: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(584283, "🌎 The Maya creation date (0.0.0.0.0)");
        // ... [rest of historical events]
        m
    };

    static ref ASTRONOMICAL_CYCLES: HashMap<&'static str, f64> = {
        let mut m = HashMap::new();
        m.insert("synodic_month", 29.530588);
        m.insert("venus_synodic", 583.92);
        m.insert("solar_year", 365.242189);
        m.insert("eclipse_year", 346.62);
        m
    };
}

// Structs and their implementations
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct LongCount {
    baktun: i32,
    katun: i32,
    tun: i32,
    uinal: i32,
    kin: i32,
}

impl LongCount {
    pub fn from_days(days: i32) -> Self {
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

    pub fn to_days(&self) -> i32 {
        self.baktun * BAKTUN_DAYS +
        self.katun * KATUN_DAYS +
        self.tun * TUN_DAYS +
        self.uinal * UINAL_DAYS +
        self.kin
    }
}

// Calendar data structure
#[derive(Clone)]
pub struct CalendarData {
    long_count: LongCount,
    tzolkin: TzolkinDate,
    haab: HaabDate,
    moon_phase: String,
    venus_phase: String,
    year_bearer: String,
    next_solstice: (String, i32),
    eclipse_status: String,
    historical_event: Option<String>,
    gregorian_date: NaiveDate,
    julian_day_number: i32,
    days_since_creation: i32,
}

// Main MayanCalendar application struct
pub struct MayanCalendar {
    current_time: chrono::NaiveTime,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: Arc<RwLock<CalendarCache>>,
    glyph_renderer: GlyphRenderer,
    calculator: ParallelCalendarCalculator,
    metrics: Arc<Metrics>,
}




// The rest of the implementations follow...
// [Note: I'll continue with more sections, but this gives you an idea of 
// how we're reorganizing the code. Would you like me to focus on any 
// particular part next?]




// UI Components Module for Mayan Calendar
use eframe::egui::{self, Context, Ui};

// First, let's create a trait that defines the basic behavior for UI components
pub trait CalendarComponent {
    fn render(&self, ui: &mut Ui);
    fn update(&mut self) -> bool; // Returns true if the component needs a repaint
}

// A struct to hold all our UI state in one place
pub struct CalendarUI {
    clock_display: ClockDisplay,
    date_display: DateDisplay,
    long_count_display: LongCountDisplay,
    calendar_round_display: CalendarRoundDisplay,
    astronomical_display: AstronomicalDisplay,
    historical_display: HistoricalDisplay,
    performance_display: PerformanceDisplay,
}

impl CalendarUI {
    pub fn new(calendar_data: &CalendarData, metrics: Arc<Metrics>) -> Self {
        Self {
            clock_display: ClockDisplay::new(),
            date_display: DateDisplay::new(calendar_data),
            long_count_display: LongCountDisplay::new(&calendar_data.long_count),
            calendar_round_display: CalendarRoundDisplay::new(&calendar_data.tzolkin, &calendar_data.haab),
            astronomical_display: AstronomicalDisplay::new(calendar_data),
            historical_display: HistoricalDisplay::new(calendar_data.historical_event.as_deref()),
            performance_display: PerformanceDisplay::new(metrics),
        }
    }

    // Main render method that coordinates all UI components
    pub fn render(&mut self, ui: &mut Ui, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Create a top panel for the clock and basic info
                self.render_top_panel(ui);
                
                // Create the main calendar display
                self.render_main_calendar(ui);
                
                // Create the bottom panel for additional info
                self.render_bottom_panel(ui);
            });
        });
    }

    // Top panel contains the clock and basic date information
    fn render_top_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.clock_display.render(ui);
            ui.add_space(20.0);
            self.date_display.render(ui);
        });
    }

    // Main calendar display contains the Long Count and Calendar Round
    fn render_main_calendar(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.long_count_display.render(ui);
            ui.add_space(10.0);
            self.calendar_round_display.render(ui);
        });
    }

    // Bottom panel contains astronomical and historical information
    fn render_bottom_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.astronomical_display.render(ui);
            ui.add_space(10.0);
            self.historical_display.render(ui);
            
            // Only show performance metrics in debug mode
            if cfg!(debug_assertions) {
                ui.add_space(20.0);
                self.performance_display.render(ui);
            }
        });
    }
}

// Individual component implementations follow. Each is designed to be
// self-contained and responsible for a specific part of the UI.

// Clock Display Component
pub struct ClockDisplay {
    current_time: chrono::NaiveTime,
}

impl ClockDisplay {
    pub fn new() -> Self {
        Self {
            current_time: chrono::Local::now().time(),
        }
    }

    pub fn update_time(&mut self, new_time: chrono::NaiveTime) {
        self.current_time = new_time;
    }
}

impl CalendarComponent for ClockDisplay {
    fn render(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
        });
    }

    fn update(&mut self) -> bool {
        let new_time = chrono::Local::now().time();
        if new_time.second() != self.current_time.second() {
            self.current_time = new_time;
            true
        } else {
            false
        }
    }
}

// Date Display Component
pub struct DateDisplay {
    gregorian_date: NaiveDate,
    days_since_creation: i32,
}

impl DateDisplay {
    pub fn new(calendar_data: &CalendarData) -> Self {
        Self {
            gregorian_date: calendar_data.gregorian_date,
            days_since_creation: calendar_data.days_since_creation,
        }
    }
}

impl CalendarComponent for DateDisplay {
    fn render(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!(
                "📅 Gregorian Date: {}", 
                self.gregorian_date.format("%Y-%m-%d")
            ));
            ui.label(format!(
                "Days since Creation: {}", 
                self.days_since_creation
            ));
        });
    }

    fn update(&mut self) -> bool {
        false // Only updates when calendar_data changes
    }
}

// Long Count Display Component - handles all Long Count representations
pub struct LongCountDisplay {
    long_count: LongCount,
    ascii_art: String,
    unicode_glyphs: String,
}

impl LongCountDisplay {
    pub fn new(long_count: &LongCount) -> Self {
        Self {
            long_count: *long_count,
            ascii_art: Self::generate_ascii_art(long_count),
            unicode_glyphs: Self::generate_unicode_glyphs(long_count),
        }
    }

    fn generate_ascii_art(long_count: &LongCount) -> String {
        // Implementation for ASCII art generation
        let mut result = String::new();
        for (value, name) in [
            (long_count.baktun, "Baktun"),
            (long_count.katun, "Katun"),
            (long_count.tun, "Tun"),
            (long_count.uinal, "Uinal"),
            (long_count.kin, "Kin"),
        ] {
            result.push_str(&format!("{}:\n{}", name, mayan_ascii_number(value)));
        }
        result
    }

    fn generate_unicode_glyphs(long_count: &LongCount) -> String {
        format!(
            "{}{}{}{}{}",
            mayan_numeral(long_count.baktun),
            mayan_numeral(long_count.katun),
            mayan_numeral(long_count.tun),
            mayan_numeral(long_count.uinal),
            mayan_numeral(long_count.kin)
        )
    }
}

impl CalendarComponent for LongCountDisplay {
    fn render(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Decimal notation
            ui.heading("Long Count Calendar");
            ui.label(format!(
                "🔢 {}.{}.{}.{}.{}",
                self.long_count.baktun,
                self.long_count.katun,
                self.long_count.tun,
                self.long_count.uinal,
                self.long_count.kin
            ));

            // Unicode glyphs
            ui.label("📜 Mayan Numerals:");
            ui.monospace(&self.unicode_glyphs);

            // ASCII art
            ui.collapsing("📝 ASCII Art Representation", |ui| {
                ui.monospace(&self.ascii_art);
            });
        });
    }

    fn update(&mut self) -> bool {
        false // Only updates when long_count changes
    }
}

// Performance Metrics Display Component
pub struct PerformanceDisplay {
    metrics: Arc<Metrics>,
}

impl PerformanceDisplay {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }
}

impl CalendarComponent for PerformanceDisplay {
    fn render(&self, ui: &mut Ui) {
        ui.collapsing("📊 Performance Metrics", |ui| {
            let report = self.metrics.report();
            ui.monospace(report);
        });
    }

    fn update(&mut self) -> bool {
        true // Always update to show latest metrics
    }
}

// Implementation for MayanCalendar struct
impl MayanCalendar {
    pub fn render(&mut self, ctx: &Context) {
        let mut calendar_ui = CalendarUI::new(&self.calendar_data, Arc::clone(&self.metrics));
        calendar_ui.render(&mut ctx.begin_frame(), ctx);
    }
}



use std::sync::Arc;
use parking_lot::RwLock;
use eframe::egui::{Context, ColorImage, TextureHandle, TextureOptions};
use image::{DynamicImage, ImageBuffer, Rgba};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

// First, let's define our glyph types more precisely
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphType {
    Tzolkin,
    Haab,
    Numeral,
    Special, // For special glyphs like period markers
}

// A struct to represent a single glyph
#[derive(Debug, Clone)]
pub struct Glyph {
    glyph_type: GlyphType,
    name: String,
    path: PathBuf,
    dimensions: (u32, u32),
    texture: Option<TextureHandle>,
}

impl Glyph {
    pub fn new(glyph_type: GlyphType, name: String, path: PathBuf) -> Self {
        Self {
            glyph_type,
            name,
            path,
            dimensions: (128, 128), // Default dimensions
            texture: None,
        }
    }
}

// A comprehensive cache for all glyph-related data
#[derive(Default)]
pub struct GlyphCache {
    tzolkin_glyphs: HashMap<String, Arc<Glyph>>,
    haab_glyphs: HashMap<String, Arc<Glyph>>,
    numeral_glyphs: HashMap<i32, Arc<Glyph>>,
    special_glyphs: HashMap<String, Arc<Glyph>>,
    loading_states: HashMap<PathBuf, bool>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_glyph(&mut self, glyph: Arc<Glyph>) {
        match glyph.glyph_type {
            GlyphType::Tzolkin => {
                self.tzolkin_glyphs.insert(glyph.name.clone(), Arc::clone(&glyph));
            }
            GlyphType::Haab => {
                self.haab_glyphs.insert(glyph.name.clone(), Arc::clone(&glyph));
            }
            GlyphType::Numeral => {
                if let Ok(num) = glyph.name.parse::<i32>() {
                    self.numeral_glyphs.insert(num, Arc::clone(&glyph));
                }
            }
            GlyphType::Special => {
                self.special_glyphs.insert(glyph.name.clone(), Arc::clone(&glyph));
            }
        }
    }

    pub fn get_glyph(&self, glyph_type: GlyphType, name: &str) -> Option<Arc<Glyph>> {
        match glyph_type {
            GlyphType::Tzolkin => self.tzolkin_glyphs.get(name).cloned(),
            GlyphType::Haab => self.haab_glyphs.get(name).cloned(),
            GlyphType::Numeral => name.parse::<i32>().ok()
                .and_then(|num| self.numeral_glyphs.get(&num).cloned()),
            GlyphType::Special => self.special_glyphs.get(name).cloned(),
        }
    }
}

// Main glyph manager that handles loading, caching, and rendering
pub struct GlyphManager {
    cache: Arc<RwLock<GlyphCache>>,
    metrics: Arc<Metrics>,
    config: Config,
    loading_queue: Arc<RwLock<Vec<PathBuf>>>,
}

impl GlyphManager {
    pub fn new(config: Config, metrics: Arc<Metrics>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(GlyphCache::new())),
            metrics,
            config,
            loading_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Preload all glyphs in parallel
    pub async fn preload_glyphs(&self, ctx: &Context) -> Result<(), GlyphError> {
        let start = Instant::now();
        
        // Collect all glyph paths from config
        let mut paths = Vec::new();
        paths.extend(self.config.tzolkin_glyphs.values().cloned());
        paths.extend(self.config.haab_glyphs.values().cloned());
        paths.extend(self.config.numeral_glyphs.values().cloned());
        
        // Process glyphs in parallel using rayon
        let results: Vec<Result<(), GlyphError>> = paths.par_iter()
            .map(|path| self.load_single_glyph(ctx, path))
            .collect();
        
        // Check for any errors
        for result in results {
            result?;
        }

        let duration = start.elapsed();
        info!(
            target: "glyph_loading",
            "Preloaded {} glyphs in {}ms",
            paths.len(),
            duration.as_millis()
        );

        Ok(())
    }

    // Load a single glyph with memory mapping
    fn load_single_glyph(&self, ctx: &Context, path: &Path) -> Result<(), GlyphError> {
        let start = Instant::now();

        // Memory map the file for efficient loading
        let file = File::open(path).map_err(GlyphError::FileError)?;
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(GlyphError::MmapError)?
        };

        // Load and process the image
        let img = image::load_from_memory(&mmap)
            .map_err(GlyphError::ImageLoadError)?;

        let (width, height) = img.dimensions();
        if width != 128 || height != 128 {
            return Err(GlyphError::InvalidDimensions(width, height));
        }

        // Convert to RGBA
        let img_rgba = img.to_rgba8();
        
        // Create texture
        let color_image = ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &img_rgba.into_raw(),
        );

        let texture = ctx.load_texture(
            path.to_string_lossy().as_ref(),
            color_image,
            TextureOptions::default(),
        );

        // Update cache
        let mut cache = self.cache.write();
        let glyph_type = self.determine_glyph_type(path);
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut glyph = Glyph::new(glyph_type, name, path.to_path_buf());
        glyph.texture = Some(texture);
        cache.insert_glyph(Arc::new(glyph));

        let duration = start.elapsed();
        self.metrics.record_glyph_load(duration);

        Ok(())
    }

    // Determine glyph type from path
    fn determine_glyph_type(&self, path: &Path) -> GlyphType {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("tzolkin") {
            GlyphType::Tzolkin
        } else if path_str.contains("haab") {
            GlyphType::Haab
        } else if path_str.contains("numeral") {
            GlyphType::Numeral
        } else {
            GlyphType::Special
        }
    }

    // Render a sequence of glyphs
    pub fn render_glyph_sequence(
        &self,
        ui: &mut Ui,
        glyphs: &[(GlyphType, String)],
        spacing: f32,
    ) {
        ui.horizontal(|ui| {
            for (glyph_type, name) in glyphs {
                if let Some(glyph) = self.cache.read().get_glyph(*glyph_type, name) {
                    if let Some(texture) = &glyph.texture {
                        ui.image(texture);
                        ui.add_space(spacing);
                    }
                }
            }
        });
    }

    // Handle failed glyph loads with fallback rendering
    fn render_fallback(&self, ui: &mut Ui, glyph_type: GlyphType, name: &str) {
        match glyph_type {
            GlyphType::Numeral => {
                if let Ok(num) = name.parse::<i32>() {
                    ui.label(format!("{}", mayan_numeral(num)));
                }
            }
            _ => {
                ui.label(format!("[{}]", name));
            }
        }
    }
}

// Enhanced error handling for glyph operations
#[derive(Debug, thiserror::Error)]
pub enum GlyphError {
    #[error("Failed to open glyph file: {0}")]
    FileError(std::io::Error),
    
    #[error("Memory mapping failed: {0}")]
    MmapError(std::io::Error),
    
    #[error("Failed to load glyph image: {0}")]
    ImageLoadError(#[from] image::ImageError),
    
    #[error("Invalid glyph dimensions: {0}x{1}, expected 128x128")]
    InvalidDimensions(u32, u32),
    
    #[error("Glyph not found: {0}")]
    GlyphNotFound(String),
}

// Extension trait for texture handling
trait TextureExt {
    fn create_placeholder(&self, ctx: &Context, size: (u32, u32)) -> TextureHandle;
}

impl TextureExt for GlyphManager {
    fn create_placeholder(&self, ctx: &Context, size: (u32, u32)) -> TextureHandle {
        let mut buffer = ImageBuffer::new(size.0, size.1);
        
        // Create a simple placeholder pattern
        for y in 0..size.1 {
            for x in 0..size.0 {
                let color = if (x + y) % 20 < 10 {
                    Rgba([200, 200, 200, 255])
                } else {
                    Rgba([150, 150, 150, 255])
                };
                buffer.put_pixel(x, y, color);
            }
        }

        let color_image = ColorImage::from_rgba_unmultiplied(
            [size.0 as usize, size.1 as usize],
            &buffer.into_raw(),
        );

        ctx.load_texture(
            "placeholder",
            color_image,
            TextureOptions::default(),
        )
    }
}



// Mayan Calendar Calculation System
use chrono::{NaiveDate, NaiveDateTime, Datelike};
use std::sync::Arc;
use parking_lot::RwLock;

// First, let's define our fundamental calendar constants
// These values form the basis of Mayan calendar mathematics
pub const TZOLKIN_CYCLE: i32 = 260;  // Length of the Tzolkin cycle (13 * 20)
pub const HAAB_CYCLE: i32 = 365;     // Length of the Haab cycle
pub const CALENDAR_ROUND: i32 = 18980; // Least common multiple of Tzolkin and Haab cycles
pub const CREATION_DATE_JDN: i32 = 584283; // Julian Day Number of the Maya creation date

// Long Count period lengths in days
pub const BAKTUN_DAYS: i32 = 144_000;  // A Baktun is 20 Katuns
pub const KATUN_DAYS: i32 = 7_200;     // A Katun is 20 Tuns
pub const TUN_DAYS: i32 = 360;         // A Tun is 18 Uinals (360 days ≈ 1 year)
pub const UINAL_DAYS: i32 = 20;        // An Uinal is 20 Kins
pub const KIN_DAYS: i32 = 1;           // A Kin is one day

/// Represents a complete Long Count date
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LongCount {
    pub baktun: i32,
    pub katun: i32,
    pub tun: i32,
    pub uinal: i32,
    pub kin: i32,
}

impl LongCount {
    /// Creates a Long Count date from the number of days since the creation date
    pub fn from_days(days: i32) -> Self {
        // We use integer division and remainder operations for maximum efficiency
        let baktun = days / BAKTUN_DAYS;
        let remainder1 = days % BAKTUN_DAYS;
        
        let katun = remainder1 / KATUN_DAYS;
        let remainder2 = remainder1 % KATUN_DAYS;
        
        let tun = remainder2 / TUN_DAYS;
        let remainder3 = remainder2 % TUN_DAYS;
        
        let uinal = remainder3 / UINAL_DAYS;
        let kin = remainder3 % UINAL_DAYS;

        Self { baktun, katun, tun, uinal, kin }
    }

    /// Converts a Long Count date back to days since creation
    pub fn to_days(&self) -> i32 {
        self.baktun * BAKTUN_DAYS +
        self.katun * KATUN_DAYS +
        self.tun * TUN_DAYS +
        self.uinal * UINAL_DAYS +
        self.kin
    }

    /// Formats the Long Count as a traditional dot-separated string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}.{}", 
            self.baktun, self.katun, self.tun, self.uinal, self.kin)
    }
}

/// Represents a date in the Tzolkin calendar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TzolkinDate {
    pub number: i32,      // Number from 1 to 13
    pub name_index: i32,  // Index from 0 to 19 for the day name
}

impl TzolkinDate {
    /// Calculates the Tzolkin date from days since creation
    pub fn from_days(days: i32) -> Self {
        // The Tzolkin calendar is a combination of two cycles:
        // - A cycle of numbers from 1 to 13
        // - A cycle of 20 day names
        
        // Calculate the number (adding 4 to align with the creation date)
        let number = ((days + 4) % 13) + 1;
        
        // Calculate the name index (adding 19 to align with the creation date)
        let name_index = (days + 19) % 20;

        Self { number, name_index }
    }

    /// Gets the Yucatec Maya name for the day
    pub fn yucatec_name(&self) -> &'static str {
        // These names are stored in order of their occurrence
        const TZOLKIN_NAMES: [&str; 20] = [
            "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
            "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
            "Chuwen", "Eb'", "B'en", "Ix", "Men",
            "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
        ];
        
        TZOLKIN_NAMES[self.name_index as usize]
    }

    /// Gets the K'iche' Maya name for the day
    pub fn kiche_name(&self) -> &'static str {
        TZOLKIN_NAMES.get(self.yucatec_name())
            .copied()
            .unwrap_or("Unknown")
    }
}

/// Represents a date in the Haab calendar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HaabDate {
    pub day: i32,        // Day number (0-19)
    pub month_index: i32, // Month index (0-18)
}

impl HaabDate {
    /// Calculates the Haab date from days since creation
    pub fn from_days(days: i32) -> Self {
        // The Haab year consists of 18 months of 20 days each,
        // plus 5 extra days (Wayeb) at the end
        
        // Calculate position in the 365-day cycle
        let year_position = days % 365;
        
        // Calculate the day and month
        let month_index = year_position / 20;
        let day = year_position % 20;

        Self { day, month_index }
    }

    /// Gets the Yucatec Maya name for the month
    pub fn month_name(&self) -> &'static str {
        const HAAB_MONTHS: [&str; 19] = [
            "Pop", "Wo'", "Sip", "Sotz'", "Sek",
            "Xul", "Yaxk'in", "Mol", "Ch'en", "Yax",
            "Sak'", "Keh", "Mak", "K'ank'in", "Muwan",
            "Pax", "K'ayab", "Kumk'u", "Wayeb'"
        ];
        
        HAAB_MONTHS[self.month_index as usize]
    }
}

/// A utility struct for astronomical calculations
pub struct AstronomicalCalculator {
    metrics: Arc<Metrics>,
}

impl AstronomicalCalculator {
    /// Calculates the moon phase for a given Julian Day Number
    pub fn moon_phase(&self, jdn: i32) -> MoonPhase {
        let start = Instant::now();
        
        // The synodic month (lunar cycle) is approximately 29.530588 days
        let synodic_month = ASTRONOMICAL_CYCLES.get("synodic_month").unwrap();
        let moon_age = (jdn as f64 % synodic_month) / synodic_month;

        let phase = match moon_age {
            x if x < 0.1 => MoonPhase::New,
            x if x < 0.25 => MoonPhase::FirstQuarter,
            x if x < 0.5 => MoonPhase::Full,
            x if x < 0.75 => MoonPhase::LastQuarter,
            _ => MoonPhase::New
        };

        self.metrics.record_calculation(start.elapsed());
        phase
    }

    /// Calculates the Venus phase for a given Julian Day Number
    pub fn venus_phase(&self, jdn: i32) -> VenusPhase {
        let start = Instant::now();
        
        // The Venus synodic period is approximately 583.92 days
        let venus_cycle = ASTRONOMICAL_CYCLES.get("venus_synodic").unwrap();
        let phase = (jdn as f64 % venus_cycle) as f64;

        let phase = match phase {
            x if x < 50.0 => VenusPhase::MorningStar,
            x if x < 215.0 => VenusPhase::SuperiorConjunction,
            x if x < 265.0 => VenusPhase::EveningStar,
            _ => VenusPhase::InferiorConjunction
        };

        self.metrics.record_calculation(start.elapsed());
        phase
    }

    /// Predicts upcoming eclipses based on the Saros cycle
    pub fn predict_eclipse(&self, jdn: i32) -> Option<EclipsePrediction> {
        let start = Instant::now();
        
        // The Saros cycle is approximately 6,585.3211 days
        const SAROS_CYCLE: f64 = 6585.3211;
        
        // Calculate days until next eclipse
        let position_in_cycle = jdn as f64 % SAROS_CYCLE;
        let days_to_next = SAROS_CYCLE - position_in_cycle;

        let prediction = if days_to_next < 15.0 {
            Some(EclipsePrediction::Lunar(days_to_next as i32))
        } else if days_to_next < 30.0 {
            Some(EclipsePrediction::Solar(days_to_next as i32))
        } else {
            None
        };

        self.metrics.record_calculation(start.elapsed());
        prediction
    }
}

/// Calendar correlation constants and conversion functions
pub mod correlation {
    /// Converts a Gregorian date to Julian Day Number
    pub fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
        // Implementation of the Julian Day Number algorithm
        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        
        day + ((153 * m + 2) / 5) + 365 * y + (y / 4) - (y / 100) + (y / 400) - 32045
    }

    /// Converts a Julian Day Number to a Gregorian date
    pub fn jdn_to_gregorian(jdn: i32) -> NaiveDate {
        let j = jdn + 32044;
        let g = j / 146097;
        let dg = j % 146097;
        let c = (dg / 36524 + 1) * 3 / 4;
        let dc = dg - c * 36524;
        let b = dc / 1461;
        let db = dc % 1461;
        let a = (db / 365 + 1) * 3 / 4;
        let da = db - a * 365;
        let y = g * 400 + c * 100 + b * 4 + a;
        let m = (da * 5 + 308) / 153 - 2;
        let d = da - (m + 4) * 153 / 5 + 122;
        let year = y - 4800 + (m + 2) / 12;
        let month = ((m + 2) % 12) + 1;
        let day = d + 1;

        NaiveDate::from_ymd_opt(year, month as u32, day as u32)
            .expect("Invalid date calculated")
    }
}

// Calendar system tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_count_conversion() {
        // Test the Maya creation date
        let creation = LongCount { baktun: 13, katun: 0, tun: 0, uinal: 0, kin: 0 };
        assert_eq!(creation.to_days(), 1872000);
        
        // Test a known historical date
        let start_classic = LongCount { baktun: 8, katun: 14, tun: 3, uinal: 1, kin: 12 };
        let days = start_classic.to_days();
        let reconstructed = LongCount::from_days(days);
        assert_eq!(start_classic, reconstructed);
    }

    #[test]
    fn test_calendar_round() {
        // Test that the Calendar Round repeats properly
        let base_date = LongCount::from_days(0);
        let tzolkin1 = TzolkinDate::from_days(0);
        let haab1 = HaabDate::from_days(0);
        
        let future_date = LongCount::from_days(CALENDAR_ROUND);
        let tzolkin2 = TzolkinDate::from_days(CALENDAR_ROUND);
        let haab2 = HaabDate::from_days(CALENDAR_ROUND);
        
        assert_eq!(tzolkin1, tzolkin2);
        assert_eq!(haab1, haab2);
    }
}



