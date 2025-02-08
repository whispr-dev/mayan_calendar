use chrono::{Local, NaiveDate, NaiveDateTime, Datelike, Timelike};
use eframe::egui::{self, ColorImage, Context, TextureHandle, TextureOptions, Ui};
use eframe::{App, Frame};
use lazy_static::lazy_static;
use lru::LruCache;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{atomic::{AtomicU64, Ordering}, Arc},
    time::Instant,
};
use tracing::{info, warn, error, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use memmap2::MmapOptions;
use std::fs::File;
use egui::IconData;

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

impl CalendarData {
    pub fn new(date: NaiveDateTime) -> Self {
        Self {
            long_count: LongCount::from_days(0),
            tzolkin: TzolkinDate::new(1, "Initial"),
            haab: HaabDate::new(1, "Initial"),
            moon_phase: String::new(),
            venus_phase: String::new(),
            year_bearer: String::new(),
            next_solstice: (String::new(), 0),
            eclipse_status: String::new(),
            historical_event: None,
            gregorian_date: date.date(),
            julian_day_number: 0,
            days_since_creation: 0,
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

// Glyph Renderer Implementation
impl GlyphRenderer {
    pub fn new(ctx: &Context, config: Config) -> Self {
        Self {
            cache: Arc::new(RwLock::new(TextureCache {
                tzolkin_textures: HashMap::new(),
                haab_textures: HashMap::new(),
            })),
            config,
            metrics: Arc::new(Metrics::new()),
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

    pub fn get_glyph_sequence(&self, glyph_specs: &[(GlyphType, String)]) -> Option<Vec<TextureHandle>> {
        let start = Instant::now();
        
        let cache = self.cache.read();
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
                let texture = match glyph_type {
                    GlyphType::Tzolkin => cache.tzolkin_textures.get(path).cloned(),
                    GlyphType::Haab => cache.haab_textures.get(path).cloned(),
                };
                
                if let Some(texture) = texture {
                    textures.push(texture);
                } else {
                    self.metrics.record_cache_miss();
                    warn!(
                        target: "glyph_rendering",
                        "Missing texture for glyph: {:?} {}",
                        glyph_type,
                        name
                    );
                    return None;
                }
            } else {
                warn!(
                    target: "glyph_rendering",
                    "No path configured for glyph: {:?} {}",
                    glyph_type,
                    name
                );
                return None;
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
        {
            let cache = self.cache.read();
            if let Some(data) = cache.get_calendar_data(days) {
                self.metrics.record_cache_hit();
                return data;
            }
        }
        self.metrics.record_cache_miss();

        // Calculate calendar components
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
        
        let current_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(); // Replace with actual current date
        let (solstice, days_to_event) = next_solstice_or_equinox(
            current_date.year(), 
            current_date.month0() as i32 + 1,  // month0() returns 0-based month 
            current_date.day() as i32
        );
        data.next_solstice = (solstice, days_to_event);
        
        data.eclipse_status = next_eclipse(days);
        data.historical_event = historical_event(days);

        // Cache the result
        let mut cache = self.cache.write();
        cache.put_calendar_data(days, data.clone());

        data
    }
}

// MayanCalendar Implementation
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

        calendar.glyph_renderer.preload_glyphs(ctx)?;

        Ok(calendar)
    }

    fn render(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_clock(ui);
                self.render_calendar(ui);
                self.render_astronomical(ui);
                self.render_historical(ui);
                
                if cfg!(debug_assertions) {
                    ui.collapsing("📊 Performance Metrics", |ui| {
                        ui.monospace(self.metrics.report());
                    });
                }
            });
        });
    }

    fn render_clock(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
        });
    }

    fn render_calendar(&mut self, ui: &mut Ui) {
        ui.heading("Long Count Calendar");
        let long_count = self.calendar_data.long_count;
        ui.label(format!(
            "🔢 {}.{}.{}.{}.{}",
            long_count.baktun, long_count.katun,
            long_count.tun, long_count.uinal, long_count.kin
        ));

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(format!(
                    "🌞 Tzolk'in: {} {}",
                    self.calendar_data.tzolkin.number,
                    self.calendar_data.tzolkin.yucatec_name
                ));
                ui.label(format!(
                    "🌙 Haab': {} {}",
                    self.calendar_data.haab.day,
                    self.calendar_data.haab.yucatec_month
                ));
            });

            // Render glyphs if available
            if let Some(glyph_sequence) = self.glyph_renderer.get_glyph_sequence(&[
                (GlyphType::Tzolkin, self.calendar_data.tzolkin.yucatec_name.to_string()),
                (GlyphType::Haab, self.calendar_data.haab.yucatec_month.to_string())
            ]) {
                ui.horizontal(|ui| {
                    for texture in glyph_sequence {
                        ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(64.0, 64.0)));
                    }
                });
            }
        });
    }

    fn render_astronomical(&self, ui: &mut Ui) {
        ui.heading("Astronomical Information");
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(format!("🌕 Moon Phase: {}", self.calendar_data.moon_phase));
                ui.label(format!("⭐ Venus Phase: {}", self.calendar_data.venus_phase));
                ui.label(format!("🌞 Year Bearer: {}", self.calendar_data.year_bearer));
                
                let (solstice, days) = &self.calendar_data.next_solstice;
                ui.label(format!("🌓 Next Event: {} (in {} days)", solstice, days));
                
                ui.label(format!("🌘 Eclipse Status: {}", self.calendar_data.eclipse_status));
            });
        });
    }

    fn render_historical(&self, ui: &mut Ui) {
        if let Some(event) = &self.calendar_data.historical_event {
            ui.heading("Historical Event");
            ui.label(format!("📜 {}", event));
        }
    }

    fn update_calendar_if_needed(&mut self) {
        let now = Local::now().naive_local();
        if now.date() != self.last_calendar_update.date() {
            let start = Instant::now();
            
            self.calendar_data = self.calculator.calculate_single_date(
                self.calendar_data.days_since_creation
            );
            
            let duration = start.elapsed();
            self.metrics.record_calculation(duration);
            
            self.last_calendar_update = now;
            
            info!(
                target: "calendar_update",
                "Updated calendar data in {}µs",
                duration.as_micros()
            );
        }
    }

    pub fn generate_performance_report(&self) -> String {
        self.metrics.report()
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logging with structured format
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
  
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_icon(include_bytes!("../assets/icon.png")),
        ..Default::default()
    };
 
    let icon_data = IconData {
        rgba: include_bytes!("c:/rust_projects/testing_ground/assets/icon.png").to_vec(),
        width: 128,  // Adjust to your icon's actual dimensions
        height: 128,
    };
    
    // Run the application
    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            
            Box::new(MayanCalendar::new(&cc.egui_ctx).expect("Failed to create Mayan Calendar"))
        })
    )
            
            match MayanCalendar::new(&cc.egui_ctx) {
                Ok(app) => Ok(Box::new(app)),
                Err(e) => {
                    error!("Failed to initialize calendar: {}", e);
                    panic!("Failed to initialize calendar: {}", e);
                }
            }
        })
    )
  }
  
  // Font configuration with Mayan numerals
  fn configure_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
  
    // Add Mayan numeral font
    fonts.font_data.insert(
        "MayanNumerals".to_owned(),
        egui::FontData::from_static(include_bytes!("c:/rust_projects/testing_ground/assets/fonts/NotoSansMayanNumerals-Regular.ttf")).into()
    );
  
    // Configure font families
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "MayanNumerals".to_owned());
  
    ctx.set_fonts(fonts);
  }
  
  // Error handling for glyph operations
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
  