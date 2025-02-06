use eframe::egui::{CentralPanel, ColorImage, Context, TextureOptions, Ui};
use eframe::App;
use std::collections::HashMap;

struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui_example(ui, ctx);
        });
    }
}

/// A function to map Tzolk'in names to their respective image file paths.
fn get_tzolkin_glyphs() -> HashMap<&'static str, &'static str> {
    let mut glyphs = HashMap::new();
    glyphs.insert("Ajaw", "C:/users/phine/desktop/tzolk'in/glyphs/ajaw.png");
    glyphs.insert("Imix", "C:/users/phine/desktop/tzolk'in/glyphs/imix.png");
    glyphs.insert("Ik'", "C:/users/phine/desktop/tzolk'in/glyphs/ik'.png");
    glyphs.insert("Ak'b'al", "C:/users/phine/desktop/tzolk'in/glyphs/ak'b'al.png");
    glyphs.insert("K'an", "C:/users/phine/desktop/tzolk'in/glyphs/ka'n.png");
    glyphs.insert("Chikchan", "C:/users/phine/desktop/tzolk'in/glyphs/chikchan.png");
    glyphs.insert("Kimi", "C:/users/phine/desktop/tzolk'in/glyphs/kimi.png");
    glyphs.insert("Manik'", "C:/users/phine/desktop/tzolk'in/glyphs/manik'.png");
    glyphs.insert("Lamat", "C:/users/phine/desktop/tzolk'in/glyphs/lamat.png");
    glyphs.insert("Muluk", "C:/users/phine/desktop/tzolk'in/glyphs/muluk.png");
    glyphs.insert("Ok", "C:/users/phine/desktop/tzolk'in/glyphs/ok.png");
    glyphs.insert("Chuwen", "C:/users/phine/desktop/tzolk'in/glyphs/chuwen.png");
    glyphs.insert("Eb'", "C:/users/phine/desktop/tzolk'in/glyphs/eb'.png");
    glyphs.insert("B'en", "C:/users/phine/desktop/tzolk'in/glyphs/b'en.png");
    glyphs.insert("Ix", "C:/users/phine/desktop/tzolk'in/glyphs/ix.png");
    glyphs.insert("Men", "C:/users/phine/desktop/tzolk'in/glyphs/men.png");
    glyphs.insert("K'ib'", "C:/users/phine/desktop/tzolk'in/glyphs/k'ib'.png");
    glyphs.insert("Kab'an", "C:/users/phine/desktop/tzolk'in/glyphs/kab'an.png");
    glyphs.insert("Etz'nab'", "C:/users/phine/desktop/tzolk'in/glyphs/etz'nab'.png");
    glyphs.insert("Kawak", "C:/users/phine/desktop/tzolk'in/glyphs/kawak'.png");
    glyphs
}

fn load_image_as_texture(ctx: &Context, path: &str) -> Result<eframe::egui::TextureHandle, String> {
    // Decode the image file into RGBA format
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img = img.to_rgba8(); // Ensure the image is in RGBA format

    let (width, height) = img.dimensions();
    if width != 128 || height != 128 {
        return Err(format!(
            "Image dimensions do not match the expected size: got {}x{}, expected 128x128.",
            width, height
        ));
    }

    // Create a ColorImage from the raw pixel data
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );

    // Load the texture into the context
    Ok(ctx.load_texture("Tzolk'in Glyph", color_image, TextureOptions::default()))
}

fn ui_example(ui: &mut Ui, ctx: &Context) {
    let tzolkin_name = "Ajaw";
    let glyphs = get_tzolkin_glyphs();

    ui.vertical(|ui| {
        ui.label("Mayan Date:");
        ui.label(format!("13 {} (Tzolk'in)", tzolkin_name));

        if let Some(image_path) = glyphs.get(tzolkin_name) {
            match load_image_as_texture(ctx, image_path) {
                Ok(texture) => {
                    ui.add_space(10.0);
                    ui.image(&texture); // Use the texture handle directly
                }
                Err(err) => {
                    ui.label(format!("❌ Failed to load image: {}", err));
                }
            }
        } else {
            ui.label("❌ No glyph found!");
        }
    });
}

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp))),
    ) {
        eprintln!("Failed to launch app: {}", e);
    }
}
