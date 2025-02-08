use rayon::prelude::*;
use memmap2::MmapOptions;
use std::fs::File;
use std::time::Instant;
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