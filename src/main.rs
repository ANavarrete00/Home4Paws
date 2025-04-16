#[path = "utils/petfinder.rs"]
mod petfinder;
use petfinder::{get_token, get_near_animals};
use eframe::egui;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read and initalize api key/secret.
    dotenv().ok();
    let client_id = match std::env::var("API_KEY") {
        Ok(val) => val,
        Err(e) => {
            eprint!("CLIENT_ID not set: {}", e);
            return Err(e.into());
        }
    };
    let client_secret = match std::env::var("API_SECRET") {
        Ok(val) => val,
        Err(e) => {
            eprint!("CLIENT_SECRET not set: {}", e);
            return Err(e.into());
        }
    };

    //get api token
    let token = get_token(&client_id, &client_secret).await?;
    
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Home4Paws",
        options,
        Box::new(|cc| Ok(Box::new(Home4PawsApp::new(cc))))
    ).map_err(|err| println!("{:?}", err));

    Ok(())
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