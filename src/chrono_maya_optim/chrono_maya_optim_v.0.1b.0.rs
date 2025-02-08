// First fix: Move the new_from_components implementation outside the struct definition
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

// Implementation block for CalendarData
impl CalendarData {
    pub fn new(date: NaiveDateTime) -> Self {
        let naive_date = date.date();
        let year = naive_date.year();
        let month = naive_date.month() as i32;
        let day = naive_date.day() as i32;
        
        let jdn = gregorian_to_jdn(year, month, day);
        let days_since_creation = jdn - CREATION_DATE_JDN;
        
        let long_count = LongCount::from_days(days_since_creation);
        let tzolkin = tzolkin_date(days_since_creation);
        let haab = haab_date(days_since_creation);
        
        let moon_phase = moon_phase(jdn).to_string();
        let venus_phase = venus_phase(jdn).to_string();
        let year_bearer = year_bearer(jdn).to_string();
        
        let (solstice_name, days_until) = next_solstice_or_equinox(year, month, day);
        let eclipse_status = next_eclipse(jdn).to_string();
        let historical_event = historical_event(jdn).map(String::from);
        
        Self {
            long_count,
            tzolkin,
            haab,
            moon_phase,
            venus_phase,
            year_bearer,
            next_solstice: (solstice_name.to_string(), days_until),
            eclipse_status,
            historical_event,
            gregorian_date: naive_date,
            julian_day_number: jdn,
            days_since_creation,
        }
    }

    pub fn new_from_components(
        long_count: LongCount,
        tzolkin: TzolkinDate,
        haab: HaabDate,
        days_since_creation: i32,
    ) -> Self {
        let jdn = days_since_creation + CREATION_DATE_JDN;
        let gregorian_date = NaiveDate::from_num_days_from_ce(jdn);
        
        Self {
            long_count,
            tzolkin,
            haab,
            moon_phase: moon_phase(jdn).to_string(),
            venus_phase: venus_phase(jdn).to_string(),
            year_bearer: year_bearer(jdn).to_string(),
            next_solstice: next_solstice_or_equinox(
                gregorian_date.year(),
                gregorian_date.month() as i32,
                gregorian_date.day() as i32,
            ),
            eclipse_status: next_eclipse(jdn).to_string(),
            historical_event: historical_event(jdn).map(String::from),
            gregorian_date,
            julian_day_number: jdn,
            days_since_creation,
        }
    }
}

// Second fix: Move ParallelCalendarCalculator outside of any impl blocks
#[derive(Default)]
pub struct ParallelCalendarCalculator {
    metrics: Arc<Metrics>,
    cache: Arc<RwLock<CalendarCache>>,
}

impl ParallelCalendarCalculator {
    pub fn new(cache: Arc<RwLock<CalendarCache>>, metrics: Arc<Metrics>) -> Self {
        Self { metrics, cache }
    }

    // Rest of the implementation remains the same...
}

// Third fix: Correct the Config implementation
pub struct Config {
    tzolkin_glyphs: HashMap<String, String>,
    haab_glyphs: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut tzolkin_glyphs = HashMap::new();
        // Add tzolkin glyph mappings
        tzolkin_glyphs.insert(
            "Imix".to_string(),
            "C:/users/phine/documents/github/mayan_calendar/src/tzolkin/glyphs/Imix.png".to_string()
        );
        // Add other tzolkin mappings...

        let mut haab_glyphs = HashMap::new();
        // Add haab glyph mappings
        haab_glyphs.insert(
            "Pop".to_string(),
            "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Pop.png".to_string()
        );
        // Add other haab mappings...

        Self {
            tzolkin_glyphs,
            haab_glyphs,
        }
    }
}