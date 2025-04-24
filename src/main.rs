#[path = "utils/petfinder.rs"]
mod petfinder;

use petfinder::get_token;
use eframe::egui;
use dotenv::dotenv;

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
    //let token = get_token(&client_id, &client_secret).await?;
    let token_status = match get_token(&client_id, &client_secret).await {
        Ok(_) => "Token retrived successfully!".to_string(),
        Err(e) => format!("Failed to get token: {}", e),
    };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Home4Paws",
        options,
        Box::new(|cc| Ok(Box::new(Home4PawsApp::new(cc, token_status))))
    ).map_err(|err| println!("{:?}", err));

    Ok(())
}

//create a fetch function after user enters location and other filters.
/*async fetch_animal() {

    Ok(())
} */

#[derive(Default)]
struct Home4PawsApp {
    status_message: String,
    location: String,
}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>, status_message: String) -> Self{
        Self{ 
            status_message, 
            location: "City/State or Zip".to_owned(),
        }
    }
}

impl eframe::App for Home4PawsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("Home4Paws - Adopt a New Friend");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.location);
                ui.button("Search");
            });
            ui.separator();
            ui.label(&self.status_message);
        });
    }
}