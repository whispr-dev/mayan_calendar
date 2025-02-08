use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Instant;
use std::fs::File;
use lru::LruCache;
use memmap2::MmapOptions;
use chrono::{NaiveDate, NaiveDateTime, Local};

use egui::viewport::ViewportBuilder;
use eframe::{App, NativeOptions};
use egui::{self, ColorImage, Context, TextureHandle, TextureOptions};
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

  // Change to take &self instead of &mut self since we're only reading
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

// Long Count Structure (from your existing code)
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

// Delete both new_from_components methods and replace with this single one in the impl CalendarData block:

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

impl GlyphRenderer {
  pub fn new(_ctx: &Context, config: Config) -> Self {
      Self {
          cache: Arc::new(RwLock::new(TextureCache {
              tzolkin_textures: HashMap::new(),
              haab_textures: HashMap::new(),
          })),
          config,
          metrics: Arc::new(Metrics::new()),
      }
  }

  fn handle_texture_update(&self, path: &str, texture: TextureHandle) -> Result<(), GlyphError> {
      let mut cache_guard = self.cache.write().map_err(|_| {
          GlyphError::FileError(std::io::Error::new(
              std::io::ErrorKind::Other,
              "Failed to acquire write lock"
          ))
      })?;
      
      if path.contains("tzolkin") {
          cache_guard.tzolkin_textures.insert(path.to_string(), texture);
      } else {
          cache_guard.haab_textures.insert(path.to_string(), texture);
      }
      Ok(())
  }    

  fn get_texture(&self, glyph_type: GlyphType, path: &str) -> Option<TextureHandle> {
      let cache_guard = self.cache.read().ok()?;
      match glyph_type {
          GlyphType::Tzolkin => cache_guard.tzolkin_textures.get(path).cloned(),
          GlyphType::Haab => cache_guard.haab_textures.get(path).cloned(),
      }
  }

  fn load_glyph_mmap(&self, path: &str) -> Result<Vec<u8>, GlyphError> {
      let start = Instant::now();
      let file = File::open(path).map_err(|e| GlyphError::FileError(e))?;
      
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

  pub fn preload_glyphs(&self, ctx: &Context) -> Result<(), GlyphError> {
      let start = Instant::now();

      let mut paths: Vec<String> = Vec::new();
      paths.extend(self.config.tzolkin_glyphs.iter().map(|(_, path)| path.clone()));
      paths.extend(self.config.haab_glyphs.iter().map(|(_, path)| path.clone()));
      
      paths.iter().try_for_each(|path| {
          match self.load_glyph_mmap(path) {
              Ok(data) => {
                  let img = image::load_from_memory(&data)
                      .map_err(GlyphError::ImageLoadError)?;
                  
                  let img = img.to_rgba8();
                  let (width, height) = img.dimensions();

                  if width != 128 || height != 128 {
                      return Err(GlyphError::InvalidDimensions(width, height));
                  }

                  let color_image = ColorImage::from_rgba_unmultiplied(
                      [width as usize, height as usize],
                      &img.into_raw(),
                  );

                  let texture = ctx.load_texture(
                      path,
                      color_image,
                      TextureOptions::default(),
                  );
                  self.handle_texture_update(path, texture)?;
                  
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

  pub fn get_glyph_sequence(&self, glyph_specs: &[(GlyphType, String)]) -> Option<Vec<TextureHandle>> {
      let start = Instant::now();
      let mut textures = Vec::with_capacity(glyph_specs.len());
      
      for (glyph_type, name) in glyph_specs {
          let path = match glyph_type {
              GlyphType::Tzolkin => {
                  self.config.tzolkin_glyphs.iter()
                      .find(|(glyph_name, _)| glyph_name == name)
                      .map(|(_, path)| path)
              },
              GlyphType::Haab => {
                  self.config.haab_glyphs.iter()
                      .find(|(glyph_name, _)| glyph_name == name)
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

// Parallel Calendar Calculator Implementation
impl ParallelCalendarCalculator {
  pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
      Self { metrics, cache }
  }

  fn get_cached_data(&self, days: i32) -> Option<CalendarData> {
      if let Ok(cache_guard) = self.cache.read() {
          cache_guard.get_calendar_data(days)
      } else {
          None
      }
  }

  fn store_cached_data(&self, days: i32, data: CalendarData) {
      if let Ok(mut cache_guard) = self.cache.write() {
          cache_guard.put_calendar_data(days, data);
      }
  }

  fn calculate_single_date(&self, days: i32) -> CalendarData {
      // Try getting from cache first
      if let Some(data) = self.get_cached_data(days) {
          self.metrics.record_cache_hit();
          return data;
      }
      self.metrics.record_cache_miss();

      // Calculate new data
      let data = self.calculate_new_data(days);
      
      // Store in cache
      self.store_cached_data(days, data.clone());
      
      data
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
      
      let current_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
      let (solstice, days_to_event) = next_solstice_or_equinox(
          current_date.year(), 
          current_date.month0() as i32 + 1,
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
        let cache = Arc::new(RwLock::new(CalendarCache::new(
            NonZeroUsize::new(1000).unwrap()
        )));
        
        let config = Config::default();
        let glyph_renderer = GlyphRenderer::new(ctx, config);
        
        // Try to preload glyphs
        if let Err(e) = glyph_renderer.preload_glyphs(ctx) {
            error!("Failed to preload glyphs: {}", e);
        }
        
        // Initialize parallel calculator
        let calculator = ParallelCalendarCalculator::new(
            Arc::clone(&cache),
            Arc::clone(&metrics)
        );
        
        let now = Local::now().naive_local();
        let calendar_data = CalendarData::new(now);
        
        Ok(Self {
            current_time: now.time(),
            calendar_data,
            last_calendar_update: now,
            cache,
            glyph_renderer,
            calculator,
            metrics,
        })
    }

    fn update_calendar_data(&mut self) {
        let now = Local::now().naive_local();
        if (now - self.last_calendar_update).num_seconds() >= 1 {
            self.current_time = now.time();
            self.calendar_data = CalendarData::new(now);
            self.last_calendar_update = now;
        }
    }

    // Add this render method to satisfy the App trait
    fn render(&mut self, ctx: &Context) {
        // Placeholder implementation - you'll want to add your actual UI rendering logic here
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Mayan Calendar");
            // Add more UI elements for displaying calendar data
        });
    }
}

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
        self.render(ctx);
    }
}

// Previous implementations for get_glyph_sequence remain the same, but modify the iterator method:
impl GlyphRenderer {
    // Modify the get_glyph_sequence method to fix the comparison issue
    pub fn get_glyph_sequence(&self, glyph_specs: &[(GlyphType, String)]) -> Option<Vec<TextureHandle>> {
        let start = Instant::now();
        let mut textures = Vec::with_capacity(glyph_specs.len());
        
        for (glyph_type, name) in glyph_specs {
            let path = match glyph_type {
                GlyphType::Tzolkin => {
                    self.config.tzolkin_glyphs.iter()
                        .find(|(glyph_name, _)| *glyph_name == *name)
                        .map(|(_, path)| path)
                },
                GlyphType::Haab => {
                    self.config.haab_glyphs.iter()
                        .find(|(glyph_name, _)| *glyph_name == *name)
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

fn load_icon() -> Result<egui::IconData, std::io::Error> {
    Ok(egui::IconData {
        rgba: vec![0; 16 * 16 * 4],  // 16x16 transparent icon
        width: 16,
        height: 16,
    })
}

fn configure_fonts(ctx: &Context) -> Result<(), Box<dyn std::error::Error>> {
  // Create a custom font collection
  let mut fonts = egui::FontDefinitions::default();

  fonts.font_data.insert(
      "NotoSansMayanNumerals-Regular.ttf".to_owned(),
      egui::FontData::from_static(include_bytes!("../assets/NotoSansMayanNumerals-Regular.ttf"))
  );

  // Customize default fonts
  fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "Arial".to_owned());
  fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "Courier New".to_owned());

  // Set size and style for different text styles
  fonts.style.text_styles = std::collections::BTreeMap::from([
      (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional)),
      (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
      (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
      (egui::TextStyle::Monospace, egui::FontId::new(12.0, egui::FontFamily::Monospace)),
  ]);

  // Install the font definitions
  ctx.set_fonts(fonts);

  Ok(())
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