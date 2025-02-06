use chrono::{Datelike, NaiveDate, Utc};  // Added NaiveDate
use eframe::egui::{CentralPanel, ColorImage, Context, TextureOptions, Ui};
use eframe::App;
use std::collections::HashMap;
use eframe::egui;

/// Convert a Gregorian date to Julian Day Number (JDN)
fn gregorian_to_jdn(year: i32, month: i32, day: i32) -> i32 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + ((153 * m + 2) / 5) + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

//// Convert a number (0-19) to a Mayan numeral Unicode character
fn mayan_numeral(n: i32) -> char {
    match n {
        0..=19 => char::from_u32(0x1D2E0 + n as u32).unwrap(),
        _ => '❓', // If out of range, return a placeholder
    }
}


//// Get Tzolk’in Day Glyphs
fn tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
  let mut glyphs = HashMap::new();
  let tzolkin_days = [
      "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
      "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
      "Chuwen", "Eb'", "B'en", "Ix", "Men",
      "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
  ];
  let tzolkin_symbols = [
      "🐊", "🌬️", "🌑", "🌽", "🐍",
      "💀", "🖐️", "🌟", "💧", "🐶",
      "🕷️", "🌾", "🌳", "🦉", "🦅",
      "🐝", "🌀", "🔪", "⛈️", "👑"
  ];
  for i in 0..20 {
      glyphs.insert(tzolkin_days[i], tzolkin_symbols[i]);
  }
  glyphs
}

//// Get Haab’ Month Glyphs
fn haab_glyphs() -> HashMap<&'static str, &'static str> {
  let mut glyphs = HashMap::new();
  let haab_months = [
      "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
      "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
      "Kayab", "Kumk'u", "Wayeb'"
  ];
  let haab_symbols = [
      "📜", "🌊", "🔥", "🦇", "🌱", "💨", "🌞", "🌧️",
      "🏺", "🌿", "❄️", "🐆", "🎭", "🔥", "🦜", "🎵",
      "🐢", "🌰", "⚠️"
  ];
  for i in 0..19 {
      glyphs.insert(haab_months[i], haab_symbols[i]);
  }
  glyphs
}

fn long_count(days: i32) -> (i32, i32, i32, i32, i32) {
    let baktun = days / 144_000;
    let rem1 = days % 144_000;
    let katun = rem1 / 7_200;
    let rem2 = rem1 % 7_200;
    let tun = rem2 / 360;
    let rem3 = rem2 % 360;
    let uinal = rem3 / 20;
    let kin = rem3 % 20;
    (baktun, katun, tun, uinal, kin)
}

/// Generate an ASCII-art Mayan Long Count representation
fn mayan_ascii_number(n: i32) -> String {
  let mut result = String::new();
  let bars = n / 5;
  let dots = n % 5;

  for _ in 0..bars {
      result.push_str("▬\n");
  }
  for _ in 0..dots {
      result.push_str("● ");
  }
  if n == 0 {
      result.push_str("𝋠"); // Mayan zero glyph
  }
  result
}

/// Get Long Count Glyphs
fn long_count_glyphs() -> HashMap<i32, &'static str> {
  let mut glyphs = HashMap::new();
  glyphs.insert(0, "🪵");  // Placeholder for Baktun glyph
  glyphs.insert(1, "🔥");
  glyphs.insert(2, "💧");
  glyphs.insert(3, "🌿");
  glyphs.insert(4, "🌞");
  glyphs.insert(5, "🌕");
  glyphs.insert(6, "🌎");
  glyphs.insert(7, "🐍");
  glyphs.insert(8, "🌪️");
  glyphs.insert(9, "⭐");
  glyphs.insert(10, "🔺");
  glyphs.insert(11, "🏹");
  glyphs.insert(12, "🌀");
  glyphs.insert(13, "🔮");
  glyphs
}

/// Get Historical Event Glyphs
fn historical_glyphs() -> HashMap<&'static str, &'static str> {
  let mut glyphs = HashMap::new();
  glyphs.insert("🌎 The Maya creation date (0.0.0.0.0)", "🌀");
  glyphs.insert("📜 Earliest Long Count Date Found", "📜");
  glyphs.insert("⚔️ Teotihuacan Influence Over Tikal Begins", "⚔️");
  glyphs.insert("🏛️ Dynasty of Copán Founded", "🏛️");
  glyphs.insert("🛑 Tikal Defeated by Calakmul", "🛑");
  glyphs.insert("👑 King Jasaw Chan K’awiil I Crowned in Tikal", "👑");
  glyphs.insert("🏰 Toltec-Maya Rule in Chichen Itzá Begins", "🏰");
  glyphs.insert("🏹 Spanish Conquer the Last Maya City, Tayasal", "🏹");
  glyphs
}

// Find a historical Mayan event for the given JDN
fn historical_event(jdn: i32) -> Option<&'static str> {
  let events = [
      (-3113, 8, 11, "🌎 The Maya creation date (0.0.0.0.0)"),
      (292, 1, 1, "📜 Earliest Long Count Date Found"),
      (378, 1, 16, "⚔️ Teotihuacan Influence Over Tikal Begins"),
      (426, 1, 1, "🏛️ Dynasty of Copán Founded"),
      (562, 1, 1, "🛑 Tikal Defeated by Calakmul"),
      (682, 6, 3, "👑 King Jasaw Chan K’awiil I Crowned in Tikal"),
      (751, 1, 1, "🏛️ Uxmal Emerges as a Major Power"),
      (869, 12, 1, "🏛️ Tikal Abandoned"),
      (987, 1, 1, "🏰 Toltec-Maya Rule in Chichen Itzá Begins"),
      (1200, 1, 1, "🔺 Decline of Chichen Itzá"),
      (1511, 8, 1, "⚔️ Spanish Make First Contact with the Maya"),
      (1697, 3, 13, "🏹 Spanish Conquer the Last Maya City, Tayasal"),
];
for (e_year, e_month, e_day, desc) in events.iter() {
  let e_jdn = gregorian_to_jdn(*e_year, *e_month, *e_day);
  if jdn == e_jdn {
      return Some(desc);
  }
}

  None
}

/// Tzolk'in Calendar: Yucatec vs. K’iche’ names
struct TzolkinDate {
  number: i32,
  yucatec_name: &'static str,
  kiche_name: &'static str,
}

fn tzolkin_date(days: i32) -> TzolkinDate {
  let number = (((days + 3) % 13 + 13) % 13) + 1;
  let yucatec_names = [
      "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
      "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
      "Chuwen", "Eb'", "B'en", "Ix", "Men",
      "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
  ];
  let kiche_names = [
      "Imox", "Iq'", "Aq'ab'al", "K'at", "Kan",
      "Kame", "Kej", "Q'anil", "Tojil", "Tz'i'",
      "B'atz'", "E", "Aj", "Ix", "Tz'ikin",
      "Ajmaq", "No'j", "Tijax", "Kawoq", "Ajpu"
  ];
  let index = (((days + 19) % 20 + 20) % 20) as usize;
  TzolkinDate {
      number,
      yucatec_name: yucatec_names[index],
      kiche_name: kiche_names[index],
  }
}

/// Haab’ Calendar: Yucatec vs. K’iche’ names
struct HaabDate {
  day: i32,
  yucatec_month: &'static str,
  kiche_month: &'static str,
}

fn haab_date(days: i32) -> HaabDate {
  let haab_day = ((days + 348) % 365 + 365) % 365;
  let month_index = haab_day / 20;
  let day = haab_day % 20;
  
  let yucatec_months = [
      "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
      "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
      "Kayab", "Kumk'u", "Wayeb'"
  ];
  
  let kiche_months = [
      "Pop", "Wo'", "Sip", "Zotz'", "Tzek", "Xul", "Yaxkin", "Mol",
      "Chen", "Yax", "Zac", "Keh", "Mak", "Kank'in", "Muwan", "Pax",
      "Kayab", "Kumk'u", "Wayeb'"
  ];
  
  let month = yucatec_months[month_index as usize];
  let kiche_month = kiche_months[month_index as usize];

  HaabDate {
      day,
      yucatec_month: month,
      kiche_month,
  }
}

/// Calculate Year Bearer (Patron Tzolk’in Day of Haab’ New Year)
fn year_bearer(jdn: i32) -> &'static str {
  let tzolkin_days = ["Ik'", "Manik'", "Eb'", "K’an"];
  let year_start_tzolkin_index = (((jdn + 348) % 260) % 4) as usize;
  tzolkin_days[year_start_tzolkin_index]
}

fn moon_phase(jdn: i32) -> &'static str {
    let synodic_month = 29.530588; // Average lunar cycle
    let moon_age = (jdn as f64 % synodic_month) / synodic_month;

    if moon_age < 0.1 {
        "🌑 New Moon"
    } else if moon_age < 0.25 {
        "🌓 First Quarter"
    } else if moon_age < 0.5 {
        "🌕 Full Moon"
    } else if moon_age < 0.75 {
        "🌗 Last Quarter"
    } else {
        "🌑 New Moon"
    }
}

/// Compute Venus Cycle Phase
fn venus_phase(jdn: i32) -> &'static str {
  let venus_cycle = 584; // Venus synodic period in days
  let phase = jdn % venus_cycle;

  if phase < 50 {
      "🌟 Morning Star (Heliacal Rise)"
  } else if phase < 215 {
      "☀️ Superior Conjunction (Invisible)"
  } else if phase < 265 {
      "⭐ Evening Star (Heliacal Set)"
  } else {
      "🌑 Inferior Conjunction (Between Earth & Sun)"
  }
}

/// Calculate upcoming solstices and equinoxes
fn next_solstice_or_equinox(year: i32, month: i32, day: i32) -> (&'static str, i32) {
  let events = [
      ("🌸 Spring Equinox", NaiveDate::from_ymd_opt(year, 3, 20).unwrap()),
      ("☀️ Summer Solstice", NaiveDate::from_ymd_opt(year, 6, 21).unwrap()),
      ("🍂 Autumn Equinox", NaiveDate::from_ymd_opt(year, 9, 22).unwrap()),
      ("❄️ Winter Solstice", NaiveDate::from_ymd_opt(year, 12, 21).unwrap()),
  ];

  let today = NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();
  
  for (name, date) in events.iter() {
      if *date >= today {
          let days_until = (*date - today).num_days() as i32;
          return (*name, days_until);
      }
  }
  
  // If past December, return next year's Spring Equinox
  ("🌸 Spring Equinox", 365 - (today.ordinal() - 79) as i32)
}

/// Predict next Lunar and Solar Eclipse
fn next_eclipse(jdn: i32) -> &'static str {
  let saros_cycle = 6585; // Average Saros cycle in days (eclipses repeat every ~18 years)
  let days_since_last_eclipse = jdn % saros_cycle;

  if days_since_last_eclipse < 15 {
      "🌑 Lunar Eclipse Soon!"
  } else if days_since_last_eclipse < 30 {
      "🌞 Solar Eclipse Soon!"
  } else {
      "🌘 No Eclipse Imminent"
  }
}

/// Check for Historical Mayan Events
fn historical_events(year: i32, month: i32, day: i32) -> Option<&'static str> {
  let events = [
      (292, 1, 1, "📜 Earliest Long Count Date Found"),
      (378, 1, 16, "🏛️ Teotihuacan Influence Over Tikal Begins"),
      (682, 6, 3, "👑 King Jasaw Chan K’awiil I Crowned in Tikal"),
      (869, 12, 1, "🏛️ Tikal Collapses"),
      (1511, 8, 1, "⚔️ Spanish Make First Contact with the Maya"),
  ];

  for (e_year, e_month, e_day, desc) in events.iter() {
      if *e_year == year && *e_month == month && *e_day == day {
          return Some(desc);
      }
  }

  None
}

// A function to map Tzolk'in names to their respective image file paths.
fn get_tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("Ajaw", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ajaw.png");
    glyphs.insert("Imix", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/imix.png");
    glyphs.insert("Ik'", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ik'.png");
    glyphs.insert("Ak'b'al", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ak'b'al.png");
    glyphs.insert("K'an", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ka'n.png");
    glyphs.insert("Chikchan", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/chikchan.png");
    glyphs.insert("Kimi", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/kimi.png");
    glyphs.insert("Manik'", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/manik'.png");
    glyphs.insert("Lamat", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/lamat.png");
    glyphs.insert("Muluk", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/muluk.png");
    glyphs.insert("Ok", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ok.png");
    glyphs.insert("Chuwen", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/chuwen.png");
    glyphs.insert("Eb'", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/eb'.png");
    glyphs.insert("B'en", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/b'en.png");
    glyphs.insert("Ix", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/ix.png");
    glyphs.insert("Men", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/men.png");
    glyphs.insert("K'ib'", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/k'ib'.png");
    glyphs.insert("Kab'an", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/kab'an.png");
    glyphs.insert("Etz'nab'", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/etz'nab'.png");
    glyphs.insert("Kawak", "C:/users/phine/documents/github/mayan_calendar/src/tzolk'in/glyphs/kawak'.png");
    glyphs
}

// A function to load Tzolk'in names as texture from image
fn load_tzolkin_image_as_texture(ctx: &Context, path: &str) -> Result<eframe::egui::TextureHandle, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    if width != 128 || height != 128 {
        return Err(format!(
            "Image dimensions do not match the expected size: got {}x{}, expected 128x128.",
            width, height
        ));
    }
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );
    Ok(ctx.load_texture("Tzolk'in Glyph", color_image, TextureOptions::default()))
}

// A function to map Haab names to their respective image file paths.
fn get_haab_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("Pop", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Pop.png");
    glyphs.insert("Wo'", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Wo'.png");
    glyphs.insert("Siq'", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Siq.png");
    glyphs.insert("Soxj'", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Soxj'.png");
    glyphs.insert("Sotj", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Sotj.png");
    glyphs.insert("Xul", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Xul.png");
    glyphs.insert("Yax'in", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Yax'in.png");
    glyphs.insert("Mal", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Mal.png");
    glyphs.insert("Chen", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Chen.png");
    glyphs.insert("Yax", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Yax.png");
    glyphs.insert("Sax", "C:/users/phine/ddocuments/github/mayan_calendar/src/haab/glyphs/Sax.png");
    glyphs.insert("Skoh", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Skoh.png");
    glyphs.insert("Mal", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Mal.png");
    glyphs.insert("Kanx'in", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Kanx'in.png");
    glyphs.insert("Muwan", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Muwan.png");
    glyphs.insert("Pax", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Pax.png");
    glyphs.insert("Kayab", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Kayab.png");
    glyphs.insert("Kunx'u", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Kunx'u.png");
    glyphs.insert("Wayeb", "C:/users/phine/documents/github/mayan_calendar/src/haab/glyphs/Wayeb.png");
    glyphs
}

// A function to load Haab names as texture from image
fn load_haab_image_as_texture(ctx: &Context, path: &str) -> Result<eframe::egui::TextureHandle, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    if width != 128 || height != 128 {
        return Err(format!(
            "Image dimensions do not match the expected size: got {}x{}, expected 128x128.",
            width, height
        ));
    }
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );
    Ok(ctx.load_texture("Haab Glyph", color_image, TextureOptions::default()))
}

fn ui_example(ui: &mut Ui, ctx: &Context) {
  let now = Utc::now().date_naive();
  let year = now.year();
  let month = now.month() as i32;
  let day = now.day() as i32;

  let jdn = gregorian_to_jdn(year, month, day);
  let days_since_creation = jdn - 584283;

  // Long Count Calculation
  let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);

  // Tzolk'in and Haab' Calendar Calculations
  let tzolkin = tzolkin_date(days_since_creation);
  let haab = haab_date(days_since_creation);

  // Additional Info
  let moon = moon_phase(jdn);
  let bearer = year_bearer(jdn);
  let venus = venus_phase(jdn);
  let (solstice, days_until) = next_solstice_or_equinox(year, month, day);
  let eclipse = next_eclipse(jdn);

  // Historical Event Lookup
  let historical = historical_event(jdn);

  // UI Layout
  ui.vertical(|ui| {
      ui.heading("Mayan Date:");

      // Gregorian Date
      ui.label(format!("📅 Gregorian Date: {}-{:02}-{:02}", year, month, day));

      // Long Count
      ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", baktun, katun, tun, uinal, kin));

      // Long Count Mayan Unicode Glyphs
      ui.label(format!(
          "📜 Long Count (Unicode): {}{}{}{}{}",
          mayan_numeral(baktun),
          mayan_numeral(katun),
          mayan_numeral(tun),
          mayan_numeral(uinal),
          mayan_numeral(kin)
      ));

      // Long Count ASCII Art
      ui.label("📜 Long Count (ASCII):");
      ui.label(format!("Baktun:\n{}", mayan_ascii_number(baktun)));
      ui.label(format!("Katun:\n{}", mayan_ascii_number(katun)));
      ui.label(format!("Tun:\n{}", mayan_ascii_number(tun)));
      ui.label(format!("Uinal:\n{}", mayan_ascii_number(uinal)));
      ui.label(format!("Kin:\n{}", mayan_ascii_number(kin)));

      // Tzolk'in and Haab' Dates
      ui.label(format!(
          "🌞 Tzolk'in Date: {} {}",
          tzolkin.number, tzolkin.yucatec_name
      ));
      ui.label(format!(
          "🌙 Haab' Date: {} {}",
          haab.day, haab.yucatec_month
      ));

      // Year Bearer
      ui.label(format!("🌞 Year Bearer: {}", bearer));

      // Moon Phase
      ui.label(format!("🌕 Moon Phase: {}", moon));

      // Venus Cycle Phase
      ui.label(format!("✨ Venus Cycle: {}", venus));

      // Solstices/Equinoxes
      ui.label(format!(
          "🌓 Next Solstice/Equinox: {} ({} days away)",
          solstice, days_until
      ));

      // Eclipse Prediction
      ui.label(format!("🌘 Eclipse Prediction: {}", eclipse));

      // Historical Events
      if let Some(event) = historical {
          ui.label(format!("🏛️ Historical Event Today: {}", event));
      } else {
          ui.label("📜 No significant historical event today.");
      }

      // Add Glyph Images for Tzolk'in and Haab'
      ui.add_space(8.0);
      ui.horizontal(|ui| {
          let tzolkin_glyphs = get_tzolkin_glyphs();
          if let Some(image_path) = tzolkin_glyphs.get(tzolkin.yucatec_name) {
              match load_tzolkin_image_as_texture(ctx, image_path) {
                  Ok(texture) => {
                      ui.image(&texture);
                  }
                  Err(err) => {
                      ui.label(format!("❌ Failed to load Tzolk'in glyph: {}", err));
                  }
              }
          }

          ui.add_space(16.0);

          let haab_glyphs = get_haab_glyphs();
          if let Some(image_path) = haab_glyphs.get(haab.yucatec_month) {
              match load_haab_image_as_texture(ctx, image_path) {
                  Ok(texture) => {
                      ui.image(&texture);
                  }
                  Err(err) => {
                      ui.label(format!("❌ Failed to load Haab' glyph: {}", err));
                  }
              }
          } else {
              ui.label("❌ No Haab' glyph found!");
          }
      });
  });
}

struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui_example(ui, ctx);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp))),
    ) {
        eprintln!("Failed to launch app: {}", e);
        let now = Utc::now().date_naive();
        let year = now.year();
        let month = now.month() as i32;
        let day = now.day() as i32;
    
        // Convert the current Gregorian date to JDN.
        let jdn = gregorian_to_jdn(year, month, day);
        // GMT correlation: 0.0.0.0.0 corresponds to JDN 584283.
        let days_since_creation = jdn - 584283;
    
        // Compute Long Count.
        let (baktun, katun, tun, uinal, kin) = long_count(days_since_creation);
    
        let tzolkin = tzolkin_date(days_since_creation);
        let haab = haab_date(days_since_creation);
    
        let jdn = gregorian_to_jdn(year, month, day);
        
        let bearer = year_bearer(jdn);
        let moon = moon_phase(jdn);
        let venus = venus_phase(jdn);
        let (solstice, days_until) = next_solstice_or_equinox(year, month, day);
        let eclipse = next_eclipse(jdn);
        let _historical = historical_events(year, month, day);
    
        // ✅ Fix: Declare glyphs before using it!
        let glyphs = long_count_glyphs();
    
        println!("📆 Gregorian Date: {}-{:02}-{:02}", year, month, day);
        println!("🔢 Julian Day Number: {}", jdn);
        println!("🕰 Days since 0.0.0.0.0: {}", days_since_creation);
        println!("🌞 Tzolk'in Date: {} {} (K’iche’: {})", tzolkin.number, tzolkin.yucatec_name, tzolkin.kiche_name);
        println!("🌙 Haab’ Date: {} {} (K’iche’: {})", haab.day, haab.yucatec_month, haab.kiche_month);
        println!("🌞 Year Bearer: {}", bearer);
        println!("🌙 Moon Phase: {}", moon);
        println!("✨ Venus Cycle: {}", venus);
        println!("🌓 Next Solstice/Equinox: {} ({} days away)", solstice, days_until);
        println!("🌘 Eclipse Prediction: {}", eclipse);
        println!("\n📜 Long Count (ASCII):");
        println!("Baktun:\n{}", mayan_ascii_number(baktun));
        println!("Katun:\n{}", mayan_ascii_number(katun));
        println!("Tun:\n{}", mayan_ascii_number(tun));
        println!("Uinal:\n{}", mayan_ascii_number(uinal));
        println!("Kin:\n{}", mayan_ascii_number(kin));
        println!(
          "📜 Long Count: {}.{}.{}.{}.{}  {}{}{}{}{} {}{}{}{}{}",
            baktun, katun, tun, uinal, kin,
            glyphs[&baktun], glyphs[&katun], glyphs[&tun], glyphs[&uinal], glyphs[&kin],
            mayan_numeral(baktun), mayan_numeral(katun), mayan_numeral(tun), 
            mayan_numeral(uinal), mayan_numeral(kin)
        );
            let _ = tzolkin_glyphs();
            let _ = haab_glyphs();
            let _ = historical_glyphs();
        
            if let Some(event) = historical_event(jdn) {  // Added missing curly brace
                let event_glyphs = historical_glyphs();
                println!("🏛️ Historical Event Today: {} {}", event, event_glyphs.get(event).unwrap_or(&"❓"));
            }
        }
    }
