// src/framebuffer.rs
use crate::color::Color;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x3377ff,
            current_color: 0xFFFFFF,
        }
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    #[inline]
    pub fn row_mut(&mut self, y: usize) -> &mut [u32] {
        let w = self.width;
        let start = y * w;
        &mut self.buffer[start..start + w]
    }

    #[inline]
    pub fn write_row(&mut self, y: usize, row: &[u32]) {
        debug_assert_eq!(row.len(), self.width);
        let dst = self.row_mut(y);
        dst.copy_from_slice(row);
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    pub fn apply_bloom(&mut self, threshold: f32, intensity: f32) {
        let mut bright_pixels = vec![Color::new(0, 0, 0); self.width * self.height];
        
        // Extraer píxeles brillantes
        for i in 0..self.buffer.len() {
            let c = Color::from_hex(self.buffer[i]);
            let luminance = (c.r as f32 * 0.299 + 
                           c.g as f32 * 0.587 + 
                           c.b as f32 * 0.114) / 255.0;
            
            if luminance > threshold {
                bright_pixels[i] = c;
            }
        }
        
        // Blur gaussiano simple (box blur)
        let radius = 5;
        let mut blurred = bright_pixels.clone();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut count = 0.0;
                
                for dy in -(radius as i32)..=(radius as i32) {
                    for dx in -(radius as i32)..=(radius as i32) {
                        let nx = (x as i32 + dx).clamp(0, self.width as i32 - 1) as usize;
                        let ny = (y as i32 + dy).clamp(0, self.height as i32 - 1) as usize;
                        let idx = ny * self.width + nx;
                        
                        r += bright_pixels[idx].r as f32;
                        g += bright_pixels[idx].g as f32;
                        b += bright_pixels[idx].b as f32;
                        count += 1.0;
                    }
                }
                
                let idx = y * self.width + x;
                blurred[idx] = Color::new(
                    (r / count) as u8,
                    (g / count) as u8,
                    (b / count) as u8,
                );
            }
        }
        
        // Combinar con imagen original
        for i in 0..self.buffer.len() {
            let original = Color::from_hex(self.buffer[i]);
            let bloom = blurred[i];
            
            let final_color = Color::new(
                ((original.r as f32 + bloom.r as f32 * intensity).min(255.0)) as u8,
                ((original.g as f32 + bloom.g as f32 * intensity).min(255.0)) as u8,
                ((original.b as f32 + bloom.b as f32 * intensity).min(255.0)) as u8,
            );
            
            self.buffer[i] = final_color.to_hex();
        }
    }

    #[inline]
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn fill(&mut self, color: u32) {
        self.buffer.fill(color);
    }

    pub fn clear(&mut self) {
        self.fill(self.background_color);
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use image::{RgbImage, Rgb};
        
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        
        for y in 0..self.height {
            for x in 0..self.width {
                let color = Color::from_hex(self.buffer[y * self.width + x]);
                img.put_pixel(x as u32, y as u32, Rgb([color.r, color.g, color.b]));
            }
        }
        
        img.save(path)?;
        Ok(())
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index] = color.to_hex();
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.buffer[index])
        } else {
            None
        }
    }
}