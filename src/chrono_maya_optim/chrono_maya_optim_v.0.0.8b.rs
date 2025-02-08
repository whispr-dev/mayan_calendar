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




fn moon_phase(jdn: i32) -> String {
  let lunar_month = ASTRONOMICAL_CYCLES["synodic_month"];
  let phase = (jdn as f64 % lunar_month) / lunar_month;
  match phase {
      p if p < 0.25 => "🌒 Waxing Crescent",
      p if p < 0.5 => "🌓 First Quarter",
      p if p < 0.75 => "🌔 Waxing Gibbous",
      p if p < 1.0 => "🌕 Full Moon",
      _ => "🌑 New Moon"
  }.to_string()
}

// Similar implementations needed for:
// - venus_phase()
// - year_bearer()
// - next_solstice_or_equinox()
// - next_eclipse()
// - historical_event()




impl CalendarData {
  pub fn new_from_components(
      long_count: LongCount,
      tzolkin: TzolkinDate,
      haab: HaabDate,
      days_since_creation: i32,
  ) -> Self {
      let jdn = days_since_creation + CREATION_DATE_JDN;
      // ... calculate other fields ...
      Self {
          long_count,
          tzolkin,
          haab,
          moon_phase: moon_phase(jdn),
          venus_phase: venus_phase(jdn),
          year_bearer: year_bearer(jdn),
          next_solstice: next_solstice_or_equinox(/* ... */),
          eclipse_status: next_eclipse(jdn),
          historical_event: historical_event(jdn).map(String::from),
          gregorian_date: /* ... */,
          julian_day_number: jdn,
          days_since_creation,
      }
  }
}




