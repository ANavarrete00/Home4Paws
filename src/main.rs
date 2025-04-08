use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Home4Paws",
        options,
        Box::new(|cc| Ok(Box::new(Home4PawsApp::new(cc))))
    ).map_err(|err| println!("{:?}", err));
}

#[derive(Default)]
struct Home4PawsApp {}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self{
        Self::default()
    }
}

impl eframe::App for Home4PawsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🐾 Home4Paws - Adopt a New Friend");
            ui.label("This will display adoptable pets from an API.");
        });
    }
}