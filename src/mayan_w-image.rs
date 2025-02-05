use eframe::egui::{CentralPanel, Context, Image, Label, Ui, Vec2};

fn ui_example(ui: &mut Ui) {
    ui.label("Mayan Date:");
    ui.label("13 Ajaw (Tzolk'in)");
    ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ajaw.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 ak'b'al (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ak'b'al.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 b'en (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/b'en.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 chikchan (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/chikchan.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 chuwen (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/chuwen.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 en' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/en'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 etz'nab' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/etz'nab'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 ik' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ik'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 imix (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/imix.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 ix (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ix.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 kab'an' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/kab'an'.png"); // Replace with actual image path
}

n ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 ka'n (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ka'n.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 kawak' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/kawak'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 k'ib' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/k'ib'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 kimi (Tzolk'in)");k
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/kimi.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 lamat (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/lamat.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 manik' (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/manik'.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 men (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/men.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 muluk (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/muluk.png"); // Replace with actual image path
}

fn ui_example(ui: &mut Ui) {
  ui.label("Mayan Date:");
  ui.label("13 ok (Tzolk'in)");
  ui.image("c:/users/phine/desktop/tzolk'in/glyphs/ok.png"); // Replace with actual image path
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(400.0, 300.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Mayan Calendar",
        options,
        Box::new(|_cc| Box::new(|ctx: &Context, _| {
            CentralPanel::default().show(ctx, ui_example);
        })),
    );
}
