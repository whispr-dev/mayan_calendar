use std::sync::Arc;
use std::sync::RwLock;
use std::num::NonZeroUsize;
use chrono::{NaiveDateTime, Datelike};

use eframe::{App, NativeOptions};
use egui::{self, Context, Vec2};
use lru::LruCache;

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
pub struct Metrics;

// ---------- DEFINE `CalendarData` STRUCT ----------

#[derive(Clone)]
pub struct CalendarData {
    long_count: LongCount,
    days_since_creation: i32,
}

impl CalendarData {
    pub fn new() -> Self {
        let long_count = LongCount::from_days(1000);
        Self {
            long_count,
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

    pub fn calculate_new_data(&self, _days: i32) -> CalendarData {
        CalendarData::new()
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
    pub fn new(ctx: &Context) -> Self {
        let metrics = Arc::new(Metrics::default());
        let cache = Arc::new(RwLock::new(CalendarCache::new(
            NonZeroUsize::new(100).unwrap(),
        )));
        let calculator = ParallelCalendarCalculator::new(Arc::clone(&cache), Arc::clone(&metrics));

        Self {
            current_time: chrono::Local::now(),
            calendar_data: CalendarData::new(),
            last_calendar_update: chrono::Local::now().naive_local(),
            cache,
            calculator,
            metrics,
        }
    }

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

    pub fn update_calendar_data(&mut self) {
        self.calendar_data = self
            .calculator
            .calculate_new_data(self.calendar_data.days_since_creation);
    }
}

// ---------- IMPLEMENT `App` FOR `MayanCalendar` ----------

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_calendar_data();
        self.render(ctx);
    }
}

// ---------- CONFIGURE FONTS ----------

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSans".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSansMayanNumerals-Regular.ttf"
        )),
    );
    ctx.set_fonts(fonts);
}

// ---------- MAIN FUNCTION ----------

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        initial_window_size: Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            Box::new(MayanCalendar::new(&cc.egui_ctx))
        }),
    )
}
