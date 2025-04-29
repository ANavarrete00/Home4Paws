use egui::{ Color32, ColorImage };
use image:: { ImageReader, load_from_memory, ColorType };
//use image::GenericImageView;

pub fn load_image_bytes(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}

pub fn load_color_image_from_bytes(bytes: &[u8]) -> Result<ColorImage, Box<dyn std::error::Error>> {
    let img = load_from_memory(bytes)?.to_rgb8();
    let size = [img.width() as usize, img.height() as usize];
    let pixels: Vec<Color32> = img.pixels().map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], 255)).collect();
    
    Ok(ColorImage{ size, pixels })
}