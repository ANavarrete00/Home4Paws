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
    let client_id = std::env::var("API_KEY")?;
    let client_secret = std::env::var("API_SECRET")?;

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

    eframe::run_native(
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
    start_up: bool,
    receiver: Option<mpsc::Receiver<Result<Vec<AnimalData>, String>>>,
    image_receiver: Vec<mpsc::Receiver<(String, Option<ColorImage>)>>,
    images_loading: std::collections::HashSet<String>,
}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>, animals: Vec<AnimalData>, token: String) -> Self{
        Self{ 
            location: "City, State or Zip".to_owned(),
            animals,
            loaded_images: HashMap::default(),
            token,
            loading: true,
            start_up: true,
            receiver: None,
            image_receiver: Vec::new(),
            images_loading: std::collections::HashSet::new(),
        }
    }

    // Draws the search bar UI and triggers animal search when clicked.
    fn draw_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.location);
            if ui.button("Search").clicked() {
                self.loading = true;
                self.start_animal_search();
            }

            if self.loading {
                ui.spinner();
            }
        });
    }

    // Sends an async request to fetch new animal data based on user input.
    fn start_animal_search (&mut self) {
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        let location = self.location.clone();
        let token = self.token.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(get_near_animals(&location, &token, 1))
                .map_err(|e| e.to_string());
            let _ = sender.send(result);
        });
    }

    // Receives and processes the animal search results from the background thread.
    fn proccess_animal_response(&mut self) {
        if let Some(reciver) = &self.receiver {
            if let Ok(result) = reciver.try_recv() {
                match result {
                    Ok(new_animals) => self.load_animal_images(new_animals),
                    Err(e) => eprintln!("Failed to fetch animals: {}", e),
                }
                self.receiver = None;
            }
        }
    }

    // Replaces the current animal list and begins loading images for the new results.
    fn load_animal_images(&mut self, new_animals: Vec<AnimalData>) {
        self.loading = true;
        self.animals = new_animals;
        self.loaded_images.clear();
        self.images_loading.clear();
        self.image_receiver.clear();

        for animal in &self.animals {
            if let Some(url) = &animal.photo_url{
                let url = url.clone();
                self.images_loading.insert(url.clone());

                let (sender, receiver) = mpsc::channel();
                self.image_receiver.push(receiver);

                std::thread::spawn(move || {
                    let result = load_image_bytes(&url)
                        .ok().and_then(|bytes| load_color_image_from_bytes(&bytes).ok());
                    let _ = sender.send((url, result));
                });
            }
        }
    }

    // Processes completed image downloads and updates the UI with textures.
    fn proccess_image_receivers(&mut self, ctx: &egui::Context) {
        //Handle results of all image receivers
        self.image_receiver.retain_mut(|receiver| {
            match receiver.try_recv() {
                Ok((url, Some(image))) => {
                    let texture = ctx.load_texture(url.clone(), image, egui::TextureOptions::default());
                    self.loaded_images.insert(url.clone(), texture);
                    self.images_loading.remove(&url);
                    false // remove this receiver from the list
                }
                Ok((url, None)) => {
                    eprintln!("Failed to load image from {}", url);
                    self.images_loading.remove(&url);
                    false // remove this receiver from the list
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,  // keep it
                Err(_) => false, // drop if disconnected
            }
        });

        if self.image_receiver.is_empty() && self.loading {
            self.loading = false;
        }
    }

    // Displays a scrollable list of animal cards in the UI.
    fn draw_animal_cards(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let animal_list = self.animals.clone();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for animal in &animal_list {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            self.draw_animal_info(ui, animal);
                            self.draw_animal_image(ui, ctx, animal);
                        });
                    });
                }

                ui.horizontal(|ui| {
                    ui.button("Prev");
                    ui.button("Next");
                });
        });
    }

    // Displays detaild information about a single animal.
    fn draw_animal_info(&self, ui: &mut egui::Ui, animal: &AnimalData) {
        //animal info for UI
        ui.vertical(|ui|{
            ui.label(format!("Name: {}", animal.name));
            ui.label(format!("Breed: {}", animal.breed));
            ui.label(format!("Age: {}", animal.age));
            ui.label(format!("Size: {}", animal.size));
            ui.label(format!("Description: {}", animal.description));
            ui.label(format!("Location: {}, {}", animal.city, animal.state));

            if animal.url != "Unkown" {
                ui.hyperlink_to("Learn more about me!", animal.url.clone());
            }
            else{
                ui.label(format!("URL: {}", animal.url));
            }
        });
    }

    // Renders an individual animal's image if it's been loaded.
    fn draw_animal_image(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, animal: &AnimalData) {
        if let Some(url) = &animal.photo_url {
            let photo_size = egui::vec2(300.0, 300.0);
            if let Some(texture) = self.loaded_images.get(url) {
                ui.add(egui::Image::new(texture).fit_to_exact_size(photo_size));
            }
        }
    }
}

impl eframe::App for Home4PawsApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.start_up {
                self.load_animal_images(self.animals.clone());
                self.start_up = false;
            }
            ui.heading("Home4Paws - Adopt a New Friend");
            self.draw_search_bar(ui);
            ui.separator();
            self.proccess_animal_response();
            self.proccess_image_receivers(ctx);
            self.draw_animal_cards(ui, ctx);
        });
    }
}