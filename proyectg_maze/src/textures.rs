// src/textures.rs
use raylib::prelude::*;
use std::collections::HashMap;
use std::slice;

pub struct TextureManager {
    images: HashMap<char, Image>,
    textures: HashMap<char, Texture2D>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut images = HashMap::new();
        let mut textures = HashMap::new();

        // Mapeo de caracteres en assets
        let texture_files = vec![
            ('+', "assets/wall1.png"),
            ('-', "assets/wall2.png"),
            ('|', "assets/wall3.png"),
            ('g', "assets/wall4.png"),
            (' ', "assets/floor.png"),
            ('c', "assets/hungrycorgi.png"),
            ('a', "assets/angricorgi.png"),
            ('b', "assets/hungrybullterrier.png"),
            ('m', "assets/hungrymastif.png"),
        ];

        for (ch, path) in texture_files {
            let image = Image::load_image(path).expect(&format!("Failed to load image {}", path));
            let texture = rl.load_texture(thread, path).expect(&format!("Failed to load texture {}", path));
            images.insert(ch, image);
            textures.insert(ch, texture);
        }

        TextureManager { images, textures }
    }

    pub fn get_pixel_color(&self, ch: char, tx: u32, ty: u32) -> Color {
        if let Some(image) = self.images.get(&ch) {
            let x = tx.min(image.width as u32 - 1) as i32;
            let y = ty.min(image.height as u32 - 1) as i32;
            get_pixel_color(image, x, y)
        } else {
            Color::WHITE
        }
    }

    pub fn get_texture(&self, ch: char) -> Option<&Texture2D> {
        self.textures.get(&ch)
    }

    pub fn get_image_size(&self, ch: char) -> (u32, u32) {
        if let Some(img) = self.images.get(&ch) {
            (img.width as u32, img.height as u32)
        } else {
            (1, 1)
        }
    }
}

fn get_pixel_color(image: &Image, x: i32, y: i32) -> Color {
    let width = image.width as usize;
    let height = image.height as usize;

    if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
        return Color::WHITE;
    }

    let x = x as usize;
    let y = y as usize;

    let data_len = width * height * 4;

    unsafe {
        let data = slice::from_raw_parts(image.data as *const u8, data_len);
        let idx = (y * width + x) * 4;

        if idx + 3 >= data_len {
            return Color::WHITE;
        }

        Color::new(data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
    }
}
