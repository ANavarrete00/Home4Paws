use egui::{ Color32, ColorImage, Context, TextureHandle };
use image::{ load_from_memory, imageops::FilterType };
use std::fs;

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

pub fn load_local_texture(ctx: &Context, path: &str, texture_name: &str, desired_size: [usize; 2]) -> Result<TextureHandle, Box<dyn std::error::Error>> {
    let image_bytes = fs::read(path)?;
    let image = image::load_from_memory(&image_bytes)?.into_rgba8();
    let resized = image::imageops::resize(&image, desired_size[0] as u32, desired_size[1] as u32, FilterType::Lanczos3);
    let size = [resized.width() as usize, resized.height() as usize];
    let pixels: Vec<Color32> = resized.pixels().map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3])).collect();

    let color_image = ColorImage{ size, pixels };
    let texture = ctx.load_texture(texture_name.to_string(), color_image, Default::default());

    Ok(texture)
}