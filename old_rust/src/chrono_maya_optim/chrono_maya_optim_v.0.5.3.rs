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

//  Performance Metrics
#[derive(Default)]
pub struct Metrics {
    calculation_time: AtomicU64,
    glyph_load_time: AtomicU64,
    render_time: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
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

pub struct GlyphRenderer {
    cache: Arc<RwLock<TextureCache>>,
    config: Config,
    metrics: Arc<Metrics>,
    ctx: Context,
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
}

pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { metrics, cache }
    }
}

// ---------- MAYAN CALENDAR STRUCT & METHODS ----------

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
    pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        let metrics = Arc::new(Metrics::new());
        let cache = Arc::new(RwLock::new(CalendarCache::new(NonZeroUsize::new(100).unwrap())));
        let config = Config::default();
        let glyph_renderer = GlyphRenderer::new(ctx, config);
        let now = chrono::Local::now().naive_local();

        Ok(Self {
            current_time: chrono::Local::now(),
            calendar_data: CalendarData::new(now),
            last_calendar_update: now,
            cache: Arc::clone(&cache),
            glyph_renderer,
            calculator: ParallelCalendarCalculator::new(Arc::clone(&cache), Arc::clone(&metrics)),
            metrics,
        })
    }

    pub fn update_calendar_data(&mut self) {
        let now = chrono::Local::now().naive_local();
        let days_since_creation = self.calendar_data.days_since_creation;
        self.calendar_data = self.calculator.calculate_new_data(days_since_creation);
        self.last_calendar_update = now;
    }

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
        });
    }
}

// ---------- IMPLEMENT APP FOR MAYAN CALENDAR ----------

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
        self.render(ctx);
    }
}

// ---------- MAIN FUNCTION ----------

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            let app = MayanCalendar::new(&cc.egui_ctx).unwrap();
            Box::new(app)
        })
    )
}
