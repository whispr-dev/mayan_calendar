// main.rs
use chrono::{Local, NaiveDate, NaiveDateTime, Datelike, Timelike, Utc};
use eframe::egui::{self, ColorImage, Context, TextureHandle, TextureOptions, Ui};
use eframe::{App, Frame};
use image::{DynamicImage, ImageBuffer, Rgba};
use lazy_static::lazy_static;
use lru::LruCache;
use memmap2::MmapOptions;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    num::NonZeroUsize,
    path::{Path, PathBuf},
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

// Load our constants and calendar data structures
include!("calendar_constants.rs");

// Main application state
pub struct MayanCalendar {
    current_time: chrono::NaiveTime,
    calendar_data: CalendarData,
    last_calendar_update: chrono::NaiveDateTime,
    cache: Arc<RwLock<CalendarCache>>,
    glyph_renderer: GlyphRenderer,
    calculator: ParallelCalendarCalculator,
    metrics: Arc<Metrics>,
}

impl MayanCalendar {
    pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize metrics and cache
        let metrics = Arc::new(Metrics::new());
        let cache = Arc::new(RwLock::new(CalendarCache::new(
            NonZeroUsize::new(100).unwrap()
        )));
        
        // Create parallel calculator
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

    fn render(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Clock display
                self.render_clock(ui);
                
                // Main calendar display
                self.render_calendar(ui);
                
                // Astronomical information
                self.render_astronomical(ui);
                
                // Historical events
                self.render_historical(ui);
                
                // Performance metrics in debug mode
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
        // Long Count display
        ui.heading("Long Count Calendar");
        let long_count = self.calendar_data.long_count;
        ui.label(format!(
            "🔢 {}.{}.{}.{}.{}",
            long_count.baktun, long_count.katun,
            long_count.tun, long_count.uinal, long_count.kin
        ));

        // Tzolkin and Haab dates
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
            self.glyph_renderer.render_glyph_sequence(
                ui,
                &[
                    (GlyphType::Tzolkin, self.calendar_data.tzolkin.yucatec_name.to_string()),
                    (GlyphType::Haab, self.calendar_data.haab.yucatec_month.to_string())
                ],
                10.0
            );
        });
    }

    fn render_astronomical(&self, ui: &mut Ui) {
        ui.heading("Astronomical Information");
        ui.label(format!("🌕 Moon Phase: {}", self.calendar_data.moon_phase));
        ui.label(format!("⭐ Venus Phase: {}", self.calendar_data.venus_phase));
        ui.label(format!("🌞 Year Bearer: {}", self.calendar_data.year_bearer));
        
        let (solstice, days) = &self.calendar_data.next_solstice;
        ui.label(format!("🌓 Next Event: {} (in {} days)", solstice, days));
        
        ui.label(format!("🌘 Eclipse Status: {}", self.calendar_data.eclipse_status));
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
            self.calendar_data = self.calculator.calculate_single_date(
                self.calendar_data.days_since_creation
            );
            self.last_calendar_update = now;
        }
    }
}

// Implement the App trait for our calendar
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

        self.render(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logging
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
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            
            match MayanCalendar::new(&cc.egui_ctx) {
                Ok(app) => Box::new(app),
                Err(e) => {
                    error!("Failed to initialize calendar: {}", e);
                    panic!("Failed to initialize calendar");
                }
            }
        })
    )
}

// Font configuration for Mayan numerals
fn configure_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Add Mayan numeral font
    fonts.font_data.insert(
        "MayanNumerals".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansMayanNumerals-Regular.ttf"))
    );

    // Configure font families
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "MayanNumerals".to_owned());

    ctx.set_fonts(fonts);
}