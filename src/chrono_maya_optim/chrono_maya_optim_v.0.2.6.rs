use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Instant;
use lru::LruCache;
use chrono::{NaiveDate, NaiveDateTime, Local, Datelike};

use egui::viewport::ViewportBuilder;
use eframe::{App, NativeOptions};
use egui::{self, Context, TextureHandle};
use tracing::{info, error};
use tracing_subscriber::EnvFilter;
use tracing::Level;

// Local module imports
mod config;
mod date_utils;
mod astronomical;
use config::Config;
use date_utils::{tzolkin_date, haab_date, TzolkinDate, HaabDate};
use astronomical::{
    moon_phase,
    venus_phase,
    year_bearer,
    next_solstice_or_equinox,
    next_eclipse,
    historical_event,
};

// Enum for Glyph Types
#[derive(Debug, Clone, Copy)]
pub enum GlyphType {
    Tzolkin,
    Haab,
}

// Performance Metrics
#[derive(Default)]
pub struct Metrics {
    calculation_time: AtomicU64,
    glyph_load_time: AtomicU64,
    render_time: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
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

// Texture Cache
pub struct TextureCache {
    tzolkin_textures: HashMap<String, TextureHandle>,
    haab_textures: HashMap<String, TextureHandle>,
}

// Calendar Cache
pub struct CalendarCache {
    cache: LruCache<i32, CalendarData>,
}

impl CalendarCache {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }

    pub fn get_calendar_data(&self, days: i32) -> Option<CalendarData> {
        self.cache.peek(&days).cloned()
    }

    pub fn put_calendar_data(&mut self, days: i32, data: CalendarData) {
        self.cache.put(days, data);
    }
}

// Glyph Error Handling
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

// Long Count Structure
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
        let baktun = days / 144_000;
        let rem1 = days % 144_000;
        let katun = rem1 / 7_200;
        let rem2 = rem1 % 7_200;
        let tun = rem2 / 360;
        let rem3 = rem2 % 360;
        let uinal = rem3 / 20;
        let kin = rem3 % 20;

        Self { baktun, katun, tun, uinal, kin }
    }

    pub fn to_days(&self) -> i32 {
        self.baktun * 144_000 +
        self.katun * 7_200 +
        self.tun * 360 +
        self.uinal * 20 +
        self.kin
    }
}

// Calendar Data Structure
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

impl CalendarData {
    pub fn new(date: NaiveDateTime) -> Self {
        // Calculate days since Mayan epoch (August 11, 3114 BCE)
        let mayan_epoch = NaiveDate::from_ymd_opt(3114, 8, 11)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
            
        let days_since_creation = date.signed_duration_since(mayan_epoch).num_days() as i32;
        let long_count = LongCount::from_days(days_since_creation);
        let tzolkin = tzolkin_date(days_since_creation);
        let haab = haab_date(days_since_creation);

        Self {
            long_count,
            tzolkin,
            haab,
            moon_phase: moon_phase(days_since_creation),
            venus_phase: venus_phase(days_since_creation),
            year_bearer: year_bearer(days_since_creation),
            next_solstice: (String::new(), 0), // Will be calculated in update
            eclipse_status: next_eclipse(days_since_creation),
            historical_event: historical_event(days_since_creation).map(|s| s.to_string()),
            gregorian_date: date.date(),
            julian_day_number: days_since_creation + 584283, // Offset to Julian Day Number
            days_since_creation,
        }
    }

    pub fn new_from_components(
        long_count: LongCount,
        tzolkin: TzolkinDate,
        haab: HaabDate,
        days: i32,
    ) -> Self {
        Self {
            long_count,
            tzolkin,
            haab,
            moon_phase: String::new(),
            venus_phase: String::new(), 
            year_bearer: String::new(),
            next_solstice: (String::new(), 0),
            eclipse_status: String::new(),
            historical_event: None,
            gregorian_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            julian_day_number: 0,
            days_since_creation: days,
        }
    }
}

// Renderer and Calculator structs
pub struct GlyphRenderer {
    cache: Arc<RwLock<TextureCache>>,
    config: Config,
    metrics: Arc<Metrics>,
}

pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

pub struct MayanCalendar {
    current_time: chrono::NaiveTime,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: Arc<RwLock<CalendarCache>>,
    glyph_renderer: GlyphRenderer,
    calculator: ParallelCalendarCalculator,
    metrics: Arc<Metrics>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { 
            metrics, 
            cache 
        }
    }

    fn calculate_new_data(&self, days: i32) -> CalendarData {
        let long_count = LongCount::from_days(days);
        let tzolkin = tzolkin_date(days);
        let haab = haab_date(days);
        
        let mut data = CalendarData::new_from_components(
            long_count,
            tzolkin,
            haab,
            days
        );

        // Populate astronomical and historical data
        data.moon_phase = moon_phase(days);
        data.venus_phase = venus_phase(days);
        data.year_bearer = year_bearer(days);
        
        let current_date = NaiveDate::from_ymd_opt(
          self.calendar_data.gregorian_date.year(),
          self.calendar_data.gregorian_date.month() as i32, 
          self.calendar_data.gregorian_date.day() as i32
      ).unwrap();
        let (solstice, days_to_event) = next_solstice_or_equinox(
            current_date.year(), 
            current_date.month() as i32,
            current_date.day() as i32
        );
        data.next_solstice = (solstice, days_to_event);
        
        data.eclipse_status = next_eclipse(days);
        data.historical_event = historical_event(days).map(|s| s.to_string());

        data
    }
}

impl MayanCalendar {
  pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
      let metrics = Arc::new(Metrics::new());
      let cache = Arc::new(RwLock::new(CalendarCache::new(NonZeroUsize::new(100).unwrap())));
      
      Ok(Self {
          current_time: chrono::NaiveTime::default(),
          calendar_data: CalendarData::new(NaiveDateTime::default()),
          last_calendar_update: NaiveDateTime::default(),
          cache: cache.clone(),
          glyph_renderer: GlyphRenderer::new(ctx, Config::default()),
          calculator: ParallelCalendarCalculator::new(cache, metrics.clone()),
          metrics,
      })
  }

    fn render(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mayan Calendar");
            
            // Display Long Count
            ui.label(format!("Long Count: {}.{}.{}.{}.{}", 
                self.calendar_data.long_count.baktun,
                self.calendar_data.long_count.katun,
                self.calendar_data.long_count.tun,
                self.calendar_data.long_count.uinal,
                self.calendar_data.long_count.kin
            ));

            // Display Tzolkin Date (using yucatec_name)
            ui.label(format!("Tzolkin: {} {}",
                self.calendar_data.tzolkin.number,
                self.calendar_data.tzolkin.yucatec_name
            ));

            // Display Haab Date (using yucatec_month)
            ui.label(format!("Haab: {} {}",
                self.calendar_data.haab.day,
                self.calendar_data.haab.yucatec_month
            ));

            // Additional astronomical information
            ui.separator();
            ui.label(format!("Moon Phase: {}", self.calendar_data.moon_phase));
            ui.label(format!("Venus Phase: {}", self.calendar_data.venus_phase));
            ui.label(format!("Year Bearer: {}", self.calendar_data.year_bearer));
            
            // Gregorian Date
            ui.separator();
            ui.label(format!("Gregorian Date: {}", self.calendar_data.gregorian_date));
        });
    }
}

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
        self.render(ctx);
    }
}

fn configure_fonts(ctx: &Context) -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom font collection
    let mut fonts = egui::FontDefinitions::default();

    // Load the Mayan Numerals font if available
    fonts.font_data.insert(
        "NotoSansMayanNumerals".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansMayanNumerals-Regular.ttf"))
    );

    // Customize default fonts
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "Arial".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "Courier New".to_owned());

    // Set text styles
    let mut text_styles = egui::Style::default().text_styles.clone();
    text_styles.insert(egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional));
    text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));
    text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional));
    text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(12.0, egui::FontFamily::Monospace));

    // Install the font definitions
    ctx.set_fonts(fonts);

    Ok(())
}

impl GlyphRenderer {
    pub fn get_glyph_sequence(&self, glyph_specs: &[(GlyphType, String)]) -> Option<Vec<TextureHandle>> {    pub fn get_glyph_sequence(&self, glyph_specs: &[(GlyphType, String)]) -> Option<Vec<TextureHandle>> {
        let start = Instant::now();
        let mut textures = Vec::with_capacity(glyph_specs.len());
        
        for (glyph_type, name) in glyph_specs {
          let path = match glyph_type {
              GlyphType::Tzolkin => {
                  self.config.tzolkin_glyphs.iter()
                  .find(|(glyph_name, _)| *glyph_name == name)
                      .map(|(_, path)| path)
              },
              GlyphType::Haab => {
                  self.config.haab_glyphs.iter()
                      .find(|(glyph_name, _)| *glyph_name == name)
                      .map(|(_, path)| path)
              }
          };
            
            if let Some(path) = path {
                if let Some(texture) = self.get_texture(*glyph_type, path) {
                    textures.push(texture);
                } else {
                    self.metrics.record_cache_miss();
                    return None;
                }
            }
        }
        
        self.metrics.record_cache_hit();
        let duration = start.elapsed();
        info!(
            target: "glyph_rendering",
            "Retrieved {} glyphs in {}µs",
            textures.len(),
            duration.as_micros()
        );
        
        Some(textures)
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logging with structured format
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact()
        .init();
 
    fn load_icon() -> Result<egui::IconData, std::io::Error> {
        Ok(egui::IconData {
            rgba: vec![0; 16 * 16 * 4],  // 16x16 transparent icon
            width: 16,
            height: 16,
        })
    }
    
    let icon = match load_icon() {
        Ok(icon) => icon,
        Err(_) => egui::IconData::default(), // Fallback to default icon on error
    };

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            let _ = configure_fonts(&cc.egui_ctx);
            let app = MayanCalendar::new(&cc.egui_ctx).unwrap();
            Box::new(app)
        })
    )
}
