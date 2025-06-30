use chrono::{Datelike, NaiveDate, Timelike, Local};
use eframe::egui::{self, Context, ScrollArea, Ui, ColorImage, TextureHandle, TextureOptions};
use eframe::{App, Frame};
use std::collections::HashMap;
use std::error::Error;
use image::io::Reader as ImageReader;

// Enhanced AssetManager with proper error handling
#[derive(Debug)]
struct AssetManager {
    tzolkin_textures: HashMap<String, TextureHandle>,
    haab_textures: HashMap<String, TextureHandle>,
    base_path: std::path::PathBuf,
}

impl AssetManager {
    fn new(ctx: &Context, base_path: &str) -> Result<Self, Box<dyn Error>> {
        let base_path = std::path::PathBuf::from(base_path);
        let mut manager = Self {
            tzolkin_textures: HashMap::new(),
            haab_textures: HashMap::new(),
            base_path,
        };

        // Load Tzolk'in glyphs
        for name in [
            "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan",
            "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
            "Chuwen", "Eb'", "B'en", "Ix", "Men",
            "Kib'", "Kab'an", "Etz'nab'", "Kawak", "Ajaw"
        ] {
            if let Ok(texture) = manager.load_glyph(ctx, name, "tzolkin") {
                manager.tzolkin_textures.insert(name.to_string(), texture);
            }
        }

        // Load Haab' glyphs
        for name in [
            "Pop", "Wo'", "Sip", "Sotz'", "Sek", "Xul", "Yaxkin", "Mol",
            "Ch'en", "Yax", "Zac", "Ceh", "Mac", "Kankin", "Muan", "Pax",
            "Kayab", "Kumk'u", "Wayeb'"
        ] {
            if let Ok(texture) = manager.load_glyph(ctx, name, "haab") {
                manager.haab_textures.insert(name.to_string(), texture);
            }
        }

        Ok(manager)
    }

    fn load_glyph(&self, ctx: &Context, name: &str, glyph_type: &str) -> Result<TextureHandle, Box<dyn Error>> {
        let file_name = format!("{}.png", name.to_lowercase().replace("'", ""));
        let path = self.base_path
            .join(glyph_type)
            .join("glyphs")
            .join(file_name);

        let img = ImageReader::open(&path)?.decode()?;
        let img = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
        let img = img.to_rgba8();
        
        let color_image = ColorImage::from_rgba_unmultiplied(
            [128, 128],
            &img.into_raw(),
        );

        Ok(ctx.load_texture(
            &format!("{}-{}", glyph_type, name),
            color_image,
            TextureOptions::default(),
        ))
    }

    fn get_tzolkin_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.tzolkin_textures.get(name)
    }

    fn get_haab_texture(&self, name: &str) -> Option<&TextureHandle> {
        self.haab_textures.get(name)
    }
}

[Previous TzolkinDate, HaabDate, and CalendarState implementations remain the same...]

struct MayanCalendar {
    state: CalendarState,
    asset_manager: AssetManager,
    current_time: chrono::NaiveTime,
}

impl MayanCalendar {
    fn new(ctx: &Context) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            state: CalendarState::new(),
            asset_manager: AssetManager::new(ctx, "C:/users/phine/documents/github/mayan_calendar/src")?,
            current_time: Local::now().time(),
        })
    }

    fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.render_calendar_side(ui);
            ui.separator();
            self.render_clock_side(ui);
        });
    }

    fn render_calendar_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Your existing calendar rendering code...
            ui.heading("Mayan Calendar");
            ui.add_space(8.0);
            
            let (baktun, katun, tun, uinal, kin) = self.state.long_count;
            ui.label(format!("🔢 Long Count: {}.{}.{}.{}.{}", 
                baktun, katun, tun, uinal, kin));
            
            ui.label(format!("🌞 Tzolk'in: {} {}", 
                self.state.tzolkin.number, self.state.tzolkin.yucatec_name));
            ui.label(format!("🌙 Haab': {} {}", 
                self.state.haab.day, self.state.haab.yucatec_month));
            
            ui.add_space(8.0);
            ui.label(&self.state.moon_phase);
            ui.label(&self.state.venus_phase);
            ui.label(format!("🌓 Next {}: {} days", 
                self.state.solstice_info.0, self.state.solstice_info.1));
            ui.label(&self.state.eclipse_prediction);
            
            if let Some(event) = &self.state.historical_event {
                ui.label(format!("🏛️ Historical Event: {}", event));
            }
        });
    }

    fn render_clock_side(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!(
                "{}:{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
                self.current_time.second()
            ));
            
            // Render glyphs in a horizontal layout
            ui.horizontal(|ui| {
                if let Some(texture) = self.asset_manager.get_tzolkin_texture(
                    self.state.tzolkin.yucatec_name
                ) {
                    ui.image(texture);
                }
                
                ui.add_space(16.0);
                
                if let Some(texture) = self.asset_manager.get_haab_texture(
                    self.state.haab.yucatec_month
                ) {
                    ui.image(texture);
                }
            });
        });
    }
}

impl App for MayanCalendar {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.current_time = Local::now().time();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render(ui);
            });
        });
        
        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|cc| {
            let app = MayanCalendar::new(&cc.egui_ctx).unwrap();
            Box::new(app) as Box<dyn App>
        }),
    )?;

    Ok(())
}