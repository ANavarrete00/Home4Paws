#[path = "utils/petfinder.rs"]
mod petfinder;
#[path = "utils/imageloader.rs"]
mod imageloader;
use std::{collections::HashMap, sync::mpsc};

use petfinder::{ get_near_animals, get_token, AnimalData };
use imageloader::{ load_image_bytes, load_color_image_from_bytes };
use eframe::egui;
use dotenv::dotenv;
use egui::{ ColorImage, TextureHandle };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //read and initalize api key/secret.
    dotenv().ok();
    let rt = tokio::runtime::Runtime::new()?;//manual runtime
    let (client_id, client_secret) = (
        std::env::var("API_KEY")?,
        std::env::var("API_SECRET")?,
    );
    /*let client_id = match std::env::var("API_KEY") {
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
    };*/

    //get api token
    let token = rt.block_on(get_token(&client_id, &client_secret))?;

    //retrive inital page of animals
    let animals = match rt.block_on(get_near_animals("Seattle, WA", &token, 1)) {
        Ok(animals) => animals,
        Err(e) => {
            eprint!("Faild to get animals: {}", e);
            Vec::new()
        },
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Home4Paws",
        options,
        Box::new(|cc| {
            Ok(Box::new(Home4PawsApp::new(cc, animals, token.clone()))) //main passes animals and token to App
        }),
    ).map_err(|err| {
        eprint!("{:?}", err);
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Eframe error")) as Box<dyn std::error::Error>
    })?;

    Ok(())
}



#[derive(Default)]
struct Home4PawsApp {
    location: String,
    animals: Vec<AnimalData>,
    loaded_images: HashMap<String, TextureHandle>,
    token: String,
    loading: bool,
    receiver: Option<mpsc::Receiver<Result<Vec<AnimalData>, String>>>,
}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>, animals: Vec<AnimalData>, token: String) -> Self{
        Self{ 
            location: "City, State or Zip".to_owned(),
            animals,
            loaded_images: HashMap::default(),
            token,
            loading: false,
            receiver: None,
        }
    }
}

impl eframe::App for Home4PawsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("Home4Paws - Adopt a New Friend");

            //Search section
            ui.horizontal(|ui| {
                //text box for searching location
                ui.text_edit_singleline(&mut self.location);
                //button to trigger search
                if ui.button("Search").clicked() {
                    //self.loading = true;
                    let (sender, receiver) = mpsc::channel();
                    self.receiver = Some(receiver);
                    self.loading = true;

                    let location = self.location.clone();
                    let token = self.token.clone();
                    
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                        let result = rt.block_on(get_near_animals(&location, &token, 1))
                            .map_err(|e| e.to_string());
                        sender.send(result).unwrap_or_else(|e| {
                            eprint!("Send error: {}", e);
                        })
                    });
                    /*let animals = rt.block_on(async{
                        get_near_animals(&location, &token, 1).await
                    });
                    
                    match animals {
                        Ok(new_animals) => {
                            self.animals = new_animals;
                            self.loaded_images.clear();
                        }
                        Err(e) => {
                            eprint!("failed to fetch animals: {}", e);
                        }
                    }*/
                };
            });

            ui.separator();

            //Handle results of async thread
            if let Some(receiver) = &self.receiver {
                if let Ok(result) = receiver.try_recv() {
                    self.loading = false;
                    match result {
                        Ok(new_animals) => {
                            self.animals = new_animals;
                            self.loaded_images.clear();
                        }
                        Err(e) => {
                            eprint!("Failed to fetch animals: {}", e);
                        }
                    }
                    self.receiver = None;
                }
            }
            
            //Indicates loading
            if self.loading {
                ui.label("Loading animals...");
            };

            //create a scroll-able area to view all the animals.
            egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui|{
                
                //groups each animal with the animals information.
                for animal in &self.animals {
                    ui.group(|ui| {        
                        ui.horizontal(|ui| {

                            //animal info for UI
                            ui.vertical(|ui|{
                                ui.label(format!("Name: {}", animal.name));
                                ui.label(format!("Breed: {}", animal.breed));
                                ui.label(format!("Description: {}", animal.description));
                            });

                            //photo section
                            if let Some(photo_url) = &animal.photo_url {
                                //a set scale for photo sizes
                                let photo_size = egui::vec2(300.0, 300.0);

                                if let Some(texture) = self.loaded_images.get(photo_url) {
                                    //ui.image(texture);
                                    ui.add(egui::Image::new(texture).fit_to_exact_size(photo_size));

                                }
                                else {
                                    if let Ok(bytes) = load_image_bytes(photo_url) {
                                        if let Ok(color_image) = load_color_image_from_bytes(&bytes) {
                                            let texture = ctx.load_texture(
                                                photo_url.clone(),
                                                color_image,
                                                egui::TextureOptions::default(),
                                            );
                                            self.loaded_images.insert(photo_url.clone(), texture.clone());
                                            ui.add(egui::Image::new(&texture).fit_to_exact_size(photo_size));
                                        }
                                    }
                                }
                            }
                        });
                    });
                }
                //buttons to change page number
                ui.horizontal_centered(|ui| {
                    ui.button("Prev"); //implement trigger to change 'page' -1 unless it is 1 already
                    ui.button("Next"); //implement trigger to change 'page' +1 unless there is not more pages.
                });
            });
            
        });
    }
}