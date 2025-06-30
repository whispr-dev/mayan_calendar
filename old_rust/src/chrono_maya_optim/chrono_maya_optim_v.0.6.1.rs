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
use tracing::error;

// Local module imports
mod config;
mod date_utils;
mod astronomical;
use config::Config;
use date_utils::{tzolkin_date, haab_date, TzolkinDate, HaabDate};
use astronomical::{moon_phase, venus_phase, year_bearer, next_solstice_or_equinox, next_eclipse, historical_event};

// ---------- DEFINE `GlyphType` ENUM ----------

#[derive(Debug, Clone, Copy)]
pub enum GlyphType {
    Tzolkin,
    Haab,
}

// ---------- DEFINE `CalendarCache` STRUCT ----------

pub struct CalendarCache {
    cache: LruCache<i32, CalendarData>,
}

impl CalendarCache {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
}

// ---------- DEFINE `LongCount` STRUCT ----------

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
}

// ---------- DEFINE `Metrics` STRUCT ----------

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
}

// ---------- DEFINE `Font` ----------

fn configure_fonts(ctx: &egui::Context) {
  let mut fonts = egui::FontDefinitions::default();
  fonts.font_data.insert(
      "NotoSans".to_owned(),
      egui::FontData::from_static(include_bytes!(
          "../assets/fonts/NotoSans-Regular.ttf"
      )),
  );
  ctx.set_fonts(fonts);
}

// ---------- DEFINE `CalendarData` STRUCT ----------

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
        let long_count = LongCount::from_days(1000);
        let tzolkin = tzolkin_date(1000);
        let haab = haab_date(1000);
        Self {
            long_count,
            tzolkin,
            haab,
            moon_phase: "Full Moon".to_string(),
            venus_phase: "Morning Star".to_string(),
            year_bearer: "Muluc".to_string(),
            next_solstice: ("Winter Solstice".to_string(), 45),
            eclipse_status: "None".to_string(),
            historical_event: Some("Mayan Historical Event".to_string()),
            gregorian_date: date.date(),
            julian_day_number: 584283,
            days_since_creation: 1000,
        }
    }
}

// ---------- DEFINE `ParallelCalendarCalculator` ----------

pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { metrics, cache }
    }

    pub fn calculate_new_data(&self, days: i32) -> CalendarData {
        CalendarData::new(chrono::Local::now().naive_local())
    }
}

// ---------- DEFINE `MayanCalendar` STRUCT ----------

pub struct MayanCalendar {
    current_time: chrono::DateTime<chrono::Local>,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: Arc<RwLock<CalendarCache>>,
    calculator: ParallelCalendarCalculator,
    metrics: Arc<Metrics>,
}

impl MayanCalendar {
  pub fn render(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Mayan Calendar");
        ui.label(format!(
            "Current Time: {}",
            self.current_time.format("%Y-%m-%d %H:%M:%S")
        ));
        ui.separator();
        ui.label("This is placeholder content.");
    });
  }
}

    pub fn update_calendar_data(&mut self) {
        let now = chrono::Local::now().naive_local();
        self.calendar_data = self.calculator.calculate_new_data(self.calendar_data.days_since_creation);
        self.last_calendar_update = now;
    }
}

impl eframe::App for MayanCalendar {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      self.render(ctx); // Call the render function
  }
}

// ---------- IMPLEMENT `App` FOR `MayanCalendar` ----------

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
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
          configure_fonts(&cc.egui_ctx); // Configure fonts
          Box::new(MayanCalendar::new(&cc.egui_ctx).unwrap())
      }),
  );  
}
