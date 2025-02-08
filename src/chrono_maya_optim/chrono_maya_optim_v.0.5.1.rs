use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use lru::LruCache;
use chrono::{NaiveDate, NaiveDateTime, Datelike};

use egui::viewport::ViewportBuilder;
use eframe::{App, NativeOptions};
use egui::{self, Context, TextureHandle, ColorImage, TextureOptions, Vec2};
use tracing::{error};
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

//  MetriPerformancecs
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

    // LruCache::get requires mutable access, so we use &mut self.
    pub fn get_calendar_data(&mut self, days: i32) -> Option<CalendarData> {
        self.cache.get(&days).cloned()
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
        // Correct Mayan epoch: August 11, 3114 BCE is represented as -3113
        let mayan_epoch = NaiveDate::from_ymd_opt(-3113, 8, 11)
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
            next_solstice: (String::new(), 0),
            eclipse_status: next_eclipse(days_since_creation),
            historical_event: historical_event(days_since_creation).map(|s| s.to_string()),
            gregorian_date: date.date(),
            julian_day_number: days_since_creation + 584283,
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

fn to_mayan_numerals(mut num: i32) -> String {
    const MAYAN_GLYPHS: [&str; 20] = [
        "\u{1D2E0}", "\u{1D2E1}", "\u{1D2E2}", "\u{1D2E3}", "\u{1D2E4}",
        "\u{1D2E5}", "\u{1D2E6}", "\u{1D2E7}", "\u{1D2E8}", "\u{1D2E9}",
        "\u{1D2EA}", "\u{1D2EB}", "\u{1D2EC}", "\u{1D2ED}", "\u{1D2EE}",
        "\u{1D2EF}", "\u{1D2F0}", "\u{1D2F1}", "\u{1D2F2}", "\u{1D2F3}",
    ];
    let mut result = Vec::new();
    while num > 0 {
        let digit = num % 20;
        result.push(MAYAN_GLYPHS[digit as usize]);
        num /= 20;
    }
    result.reverse();
    result.join(" ")
}

pub struct GlyphRenderer {
    cache: Arc<RwLock<TextureCache>>,
    config: Config,
    metrics: Arc<Metrics>,
    ctx: Context, // Egui context
}

impl GlyphRenderer {
    pub fn new(ctx: &Context, config: Config) -> Self {
        Self {
            cache: Arc::new(RwLock::new(TextureCache {
                tzolkin_textures: HashMap::new(),
                haab_textures: HashMap::new(),
            })),
            config,
            metrics: Arc::new(Metrics::new()),
            ctx: ctx.clone(),
        }
    }

    pub fn get_texture(&self, glyph_type: GlyphType, name: &str) -> Option<TextureHandle> {
        // Normalize the key to match the config
        let name_lowercase = name.to_lowercase();

        // Fetch the correct glyph path
        let path_opt = match glyph_type {
            GlyphType::Tzolkin => self.config.tzolkin_glyphs.get(&name_lowercase),
            GlyphType::Haab => self.config.haab_glyphs.get(&name_lowercase),
        };

        let path = match path_opt {
            Some(p) => p,
            None => {
                error!("Glyph not found for: {}", name);
                return None;
            }
        };

        // Ensure the file actually exists
        if !Path::new(path).exists() {
            error!("Glyph file does not exist: {}", path);
            return None;
        }

        // Lock cache for reading
        let mut cache = self.cache.write().unwrap();
        if let Some(texture) = match glyph_type {
            GlyphType::Tzolkin => cache.tzolkin_textures.get(path).cloned(),
            GlyphType::Haab => cache.haab_textures.get(path).cloned(),
        } {
            return Some(texture);
        }

        // Load the image
        let image = match image::open(path) {
            Ok(img) => img,
            Err(e) => {
                error!("Failed to load image at {}: {}", path, e);
                return None;
            }
        };

        let size = [image.width() as usize, image.height() as usize];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let image_data = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        // Load texture into egui
        let texture = self.ctx.load_texture(name, image_data, TextureOptions::default());

        // Store in cache
        match glyph_type {
            GlyphType::Tzolkin => cache.tzolkin_textures.insert(path.clone(), texture.clone()),
            GlyphType::Haab => cache.haab_textures.insert(path.clone(), texture.clone()),
        };

        Some(texture)
    }
}

pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { metrics, cache }
    }
    
    pub fn calculate_new_data(&self, days: i32) -> CalendarData {
        let long_count = LongCount::from_days(days);
        let tzolkin = tzolkin_date(days);
        let haab = haab_date(days);
        let mut data = CalendarData::new_from_components(long_count, tzolkin, haab, days);
        // Populate astronomical and historical data...
        // (Dummy updates for now)
        data.moon_phase = moon_phase(days);
        data.venus_phase = venus_phase(days);
        data.year_bearer = year_bearer(days);
        // Use the current date from the system
        let current_date = chrono::Local::now().naive_local().date();
        let (solstice, days_to_event) = next_solstice_or_equinox(
            current_date.year(), 
            current_date.month() as i32,
            current_date.day() as i32
        );
        data.next_solstice = (solstice, days_to_event);
        data.eclipse_status = next_eclipse(days);
        data.historical_event = historical_event(days).map(|s| s.to_string());
         // Update the Gregorian date to the current date
        data.gregorian_date = current_date;
        data.julian_day_number = days + 584283;
        data
    }
}

// ----- MAYAN CALENDAR STRUCT & METHODS -----

pub struct MayanCalendar {
    current_time: chrono::DateTime<chrono::Local>,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: Arc<RwLock<CalendarCache>>,
    glyph_renderer: GlyphRenderer,
    calculator: ParallelCalendarCalculator,
    metrics: Arc<Metrics>,
}

impl MayanCalendar {
  pub fn render(&mut self, ctx: &Context) {
      let desired_size = Vec2::new(128.0, 128.0);
      egui::CentralPanel::default().show(ctx, |ui| {
          ui.heading("Mayan Calendar");

          // Fetch Haab glyph texture
          let haab_name = self.calendar_data.haab.yucatec_month.to_lowercase();
          if let Some(haab_glyph) = self.glyph_renderer.get_texture(GlyphType::Haab, &haab_name) {
              let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
              ui.painter().image(
                  haab_glyph.id(),
                  rect,
                  egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                  egui::Color32::WHITE,
              );
          } else {
              ui.label(format!("Haab Glyph Not Found: {}", haab_name));
          }

          // Fetch Tzolk'in glyph texture
          let tzolkin_name = self.calendar_data.tzolkin.yucatec_name.to_lowercase();
          if let Some(tzolkin_glyph) = self.glyph_renderer.get_texture(GlyphType::Tzolkin, &tzolkin_name) {
              let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
              ui.painter().image(
                  tzolkin_glyph.id(),
                  rect,
                  egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                  egui::Color32::WHITE,
              );
          } else {
              ui.label(format!("Tzolk'in Glyph Not Found: {}", tzolkin_name));
          }

          ui.separator();
          ui.label(format!("Current Time: {}", self.current_time.format("%Y-%m-%d %H:%M:%S")));
          ui.label(format!(
              "Long Count: {}.{}.{}.{}.{}", 
              self.calendar_data.long_count.baktun,
              self.calendar_data.long_count.katun,
              self.calendar_data.long_count.tun,
              self.calendar_data.long_count.uinal,
              self.calendar_data.long_count.kin
          ));
          ui.label(format!(
              "Tzolkin: {} {}",
              self.calendar_data.tzolkin.number,
              self.calendar_data.tzolkin.yucatec_name
          ));
          ui.label(format!(
              "Haab: {} {}",
              self.calendar_data.haab.day,
              self.calendar_data.haab.yucatec_month
          ));
          ui.separator();
      });
  }
}

// ----- IMPLEMENT APP FOR MAYAN CALENDAR -----

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
        self.render(ctx);
    }
}

// ----- CONFIGURE FONTS & MAIN -----

fn configure_fonts(ctx: &Context) -> Result<(), Box<dyn std::error::Error>> {
    let mut fonts = egui::FontDefinitions::default();
    // Load the Mayan Numerals font
    fonts.font_data.insert(
        "NotoSansMayanNumerals".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansMayanNumerals-Regular.ttf")),
    );
    // Assign it to a style
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "NotoSansMayanNumerals".to_owned());
    ctx.set_fonts(fonts);
    Ok(())
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logging with structured format.
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact()
        .init();

    fn load_icon() -> Result<egui::IconData, std::io::Error> {
        Ok(egui::IconData {
            rgba: vec![0; 16 * 16 * 4], // 16x16 transparent icon.
            width: 16,
            height: 16,
        })
    }
    
    let icon = match load_icon() {
        Ok(icon) => icon,
        Err(_) => egui::IconData::default(), // Fallback to default icon on error.
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
