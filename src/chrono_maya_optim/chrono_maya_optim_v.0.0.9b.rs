impl CalendarData {
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




[dependencies]
chrono = "0.4"
lazy_static = "1.4"




