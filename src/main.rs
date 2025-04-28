#[path = "utils/petfinder.rs"]
mod petfinder;

use std::fmt::format;

use petfinder::{get_near_animals, get_token, AnimalData};
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
    let token = get_token(&client_id, &client_secret).await?;

    //retrive inital page of animals
    let animals = match get_near_animals("Seattle, WA", &token, 1).await{
        Ok(animals) => animals,
        Err(e) => {
            eprint!("Faild to get animals: {}", e);
            Vec::new()
        },
    };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Home4Paws",
        options,
        Box::new(|cc| Ok(Box::new(Home4PawsApp::new(cc, animals))))
    ).map_err(|err| println!("{:?}", err));

    Ok(())
}



#[derive(Default)]
struct Home4PawsApp {
    location: String,
    animals: Vec<AnimalData>,
}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>, animals: Vec<AnimalData>) -> Self{
        Self{ 
            location: "City/State or Zip".to_owned(),
            animals,
        }
    }
}

impl eframe::App for Home4PawsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("Home4Paws - Adopt a New Friend");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.location);
                ui.button("Search");/*.clicked(){
                    //pass location to function in petfinder.rs to retrive pets.
                    if(self.location.is_empty())
                    {
                        //error message here
                    }
                    else
                    {
                        animal_data = get_near_animals(location, token, page);
                    };
                };*/
            });
            ui.separator();

            //create a scroll-able area to view all the animals.
            egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui|{
                //groups each animal with the animals information.
                for animal in &self.animals {
                    ui.group(|ui| {
                        ui.label(format!("Name: {}", animal.name));
                        ui.label(format!("Breed: {}", animal.breed));
                        ui.label(format!("Description: {}", animal.description));
                    });
                }
                ui.horizontal_centered(|ui| {
                    ui.button("Prev");
                    ui.button("Next");
                });
            });
            
        });
    }
}