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
    let location = "San Diego, CA".to_string();
    let api_page = 1;

    //retrive inital page of animals
    let animals = match rt.block_on(get_near_animals(&location, &token, &api_page)) {
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
    animal_list1: Vec<AnimalData>,
    animal_list2: Vec<AnimalData>,
    loaded_images: HashMap<String, TextureHandle>,
    token: String,
    api_page: u32,
    app_page: u32,
    list_index: i32,
    loading: bool,
    start_up: bool,
    scroll_to_top: bool,
    receiver: Option<mpsc::Receiver<Result<Vec<AnimalData>, String>>>,
    image_receiver: Vec<mpsc::Receiver<(String, Option<ColorImage>)>>,
    images_loading: std::collections::HashSet<String>,
}

impl Home4PawsApp {
    fn new(_cc: &eframe::CreationContext<'_>, animals: Vec<AnimalData>, token: String) -> Self{
        Self{ 
            location: Default::default(),
            animals,
            animal_list1: Vec::new(),
            animal_list2: Vec::new(),
            loaded_images: HashMap::default(),
            token,
            api_page: 1,
            app_page: 1,
            list_index: 0,
            loading: true,
            start_up: true,
            scroll_to_top: false,
            receiver: None,
            image_receiver: Vec::new(),
            images_loading: std::collections::HashSet::new(),
        }
    }

    // Draws the search bar UI and triggers animal search when clicked.
    fn draw_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
            let window_width = ui.available_width();
            let search_width = 400.0;
            let pad = (window_width - search_width).max(0.0) /2.0;
                ui.horizontal_centered(|ui| {
                    ui.add_space(pad);
                    ui.add(egui::TextEdit::singleline(&mut self.location).hint_text("City, State or Zip Code"));
                    if ui.button("Search").clicked() {
                        self.loading = true;
                        self.app_page = 1;
                        self.api_page = 1;
                        self.start_animal_search();
                    }

                    if self.loading {
                        ui.spinner();
                    }
                });
        });
    }

    // Sends an async request to fetch new animal data based on user input.
    fn start_animal_search (&mut self) {
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.loading = true;

        let mut location = self.location.clone();
        let token = self.token.clone();
        let api_page = self.api_page;

        if location == "" { // Empty TextEdit box will default to San Diego
            location = "San Diego, CA".to_string();
        }

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(get_near_animals(&location, &token, &api_page))
                .map_err(|e| e.to_string());
            let _ = sender.send(result);
        });
    }

    // Receives and processes the animal search results from the background thread.
    fn proccess_animal_response(&mut self) {
        if let Some(reciver) = &self.receiver {
            if let Ok(result) = reciver.try_recv() {
                println!("Location: '{}', page: {}", self.location, self.api_page);
                match result {
                    Ok(new_animals) => self.load_animal_images(new_animals),
                    Err(e) => eprintln!("Failed to fetch animals: {}", e),
                }
                self.receiver = None;
            }
        }
        self.animal_list1.clear();
        self.animal_list2.clear();
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
    fn draw_animal_cards(&mut self, ui: &mut egui::Ui) {
        let split = self.animals.len() / 2;
        self.animal_list1 = self.animals.clone();
        self.animal_list2 = self.animal_list1.split_off(split);
        let photo_size = ui.available_width() * 0.2;

        if self.app_page % 2 == 1 && self.list_index == 0 {  // Returns first half of animals in animal Vec.
            for animal in self.animal_list1.clone() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        self.draw_animal_info(ui, &animal);
                        self.draw_animal_image(ui, &animal, photo_size);
                    });
                });
            }
        }
        else if self.app_page % 2 == 0 && self.list_index == 1 {  // Returns second half of animals in animal Vec.
            for animal in self.animal_list2.clone() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        self.draw_animal_info(ui, &animal);
                        self.draw_animal_image(ui, &animal, photo_size);
                    });
                });
            }
        }
    }

    // Displays page navigation and triggers animal search when needed.
    fn draw_page_nav(&mut self, ui: &mut egui::Ui) {
        let window_width = ui.available_width();
        let nav_width = 150.0;
        let pad = (window_width - nav_width).max(0.0) / 2.0;

        ui.horizontal(|ui| {
            ui.add_space(pad); // padding to center page nav section
            
            if ui.button("Prev").clicked() { // Button for user to return to previous page of animals

                if self.app_page > 1 {
                    self.app_page -= 1;
                    self.list_index -= 1;

                    if self.list_index < 0 { // Triggers API to get previous page 
                        self.list_index = 1;
                        self.api_page -= 1;
                        self.start_animal_search();
                    }
                }
                self.scroll_to_top = true; // Scroll to top of scroll area
            };

            ui.label(format!("page {}", self.app_page)); // Display current page number

            if ui.button("Next").clicked() { // Button for user to return to previous page of animals
                self.app_page += 1;
                self.list_index += 1;

                if self.list_index > 1 { // Triggers API to get next page once there are no more animals to view in animal Vec.
                    self.list_index = 0;
                    self.api_page += 1;
                    self.start_animal_search();
                }
                self.scroll_to_top = true; // Scroll to top of scroll area
            };
        });
    }

    // Displays detaild information about a single animal.
    fn draw_animal_info(&self, ui: &mut egui::Ui, animal: &AnimalData) {
        //animal info for UI
        ui.vertical(|ui|{
            ui.label(format!("NAME: {}", animal.name));
            ui.label(format!("BREED: {}", animal.breed));
            ui.label(format!("AGE: {}", animal.age));
            ui.label(format!("SIZE: {}", animal.size));
            if !animal.good_with.is_empty(){
                ui.label(format!("{}", animal.good_with));
            } 
            ui.label(format!("DESCRIPTION: {}", animal.description));
            ui.label(format!("LOCATION: {}, {}", animal.city, animal.state));

            if animal.url != "Unkown" {
                ui.hyperlink_to("Learn more about me!", animal.url.clone());
            }
            else{
                ui.label(format!("URL: {}", animal.url));
            }
        });
    }

    // Renders an individual animal's image if it's been loaded.
    fn draw_animal_image(&mut self, ui: &mut egui::Ui, animal: &AnimalData, photo_size: f32) {
        if let Some(url) = &animal.photo_url {
            let scaled_photo_size = egui::vec2(photo_size, photo_size);
            if let Some(texture) = self.loaded_images.get(url) {
                ui.add(egui::Image::new(texture).fit_to_exact_size(scaled_photo_size));
            }
        }
    }
}

impl eframe::App for Home4PawsApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Home4Paws - Adopt a New Friend");
            self.draw_search_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.start_up {
                self.load_animal_images(self.animals.clone());
                self.start_up = false;
            }
            self.proccess_animal_response();
            self.proccess_image_receivers(ctx);
            egui::ScrollArea::vertical().id_salt("animal_scroll_area").auto_shrink([false; 2]).show(ui, |ui| { // Scroll area for animal cards

                if self.scroll_to_top {
                    ui.scroll_to_cursor(Some(egui::Align::TOP));
                    self.scroll_to_top = false;
                }
                self.draw_page_nav(ui);
                self.draw_animal_cards(ui);
                ui.separator();
                self.draw_page_nav(ui);
            });
        });
    }
}