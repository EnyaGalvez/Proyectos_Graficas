// src/stars.rs
use nalgebra_glm::Vec3;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Stars {
    star_pixels: Vec<(usize, usize, u8)>
}

impl Stars {
    pub fn new(count: usize, width: usize, height: usize, camera: &Camera) -> Self {
        let mut star_pixels = Vec::with_capacity(count);
        
        let mut seed = 12345u32;
        
        for _ in 0..count {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r1 = (seed as f64 / u32::MAX as f64) as f32;
            
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r2 = (seed as f64 / u32::MAX as f64) as f32;
            
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r3 = (seed as f64 / u32::MAX as f64) as f32;
            
            let theta = r1 * 2.0 * PI;
            let phi = (2.0 * r2 - 1.0).acos();
            
            // Convertir a coordenadas cartesianas (vector dirección)
            let sin_phi = phi.sin();
            let direction = Vec3::new(
                sin_phi * theta.cos(),
                sin_phi * theta.sin(),
                phi.cos(),
            ).normalize();
            
            // Proyectar dirección a coordenadas de pantalla con brillo variable
            if let Some((x, y)) = Self::project_to_screen(&direction, camera, width, height) {
                let brightness = if r3 > 0.9 {
                    255
                } else if r3 > 0.7 {
                    200
                } else {
                    150
                };
                
                star_pixels.push((x, y, brightness));
            }
        }
        
        Stars { star_pixels }
    }
    
    // Proyectar un vector 3D a coordenadas de pantalla 2D
    fn project_to_screen(direction: &Vec3, camera: &Camera, width: usize, height: usize) -> Option<(usize, usize)> {
        let forward = (camera.center - camera.eye).normalize();
        let right = forward.cross(&camera.up).normalize();
        let up = right.cross(&forward);
        
        let dot_forward = direction.dot(&forward);
        if dot_forward <= 0.0 {
            return None;
        }
        
        let dot_right = direction.dot(&right);
        let dot_up = direction.dot(&up);
        
        // FOV de la cámara (π/3 = 60 grados)
        let fov = PI / 3.0;
        let aspect_ratio = width as f32 / height as f32;
        
        let screen_x = dot_right / (dot_forward * (fov / 2.0).tan() * aspect_ratio);
        let screen_y = -dot_up / (dot_forward * (fov / 2.0).tan());
        
        if screen_x.abs() > 1.0 || screen_y.abs() > 1.0 {
            return None;
        }
        
        // Convertir a coordenadas de píxel
        let pixel_x = ((screen_x + 1.0) * 0.5 * width as f32) as usize;
        let pixel_y = ((screen_y + 1.0) * 0.5 * height as f32) as usize;
        
        if pixel_x < width && pixel_y < height {
            Some((pixel_x, pixel_y))
        } else {
            None
        }
    }
    
    // Dibujar todas las estrellas directamente en el framebuffer
    pub fn draw_to_framebuffer(&self, framebuffer: &mut Framebuffer) {
        for &(x, y, brightness) in &self.star_pixels {
            framebuffer.set_pixel(x, y, Color::new(brightness, brightness, brightness));
        }
    }
    
    // Mantener sample() para compatibilidad (retorna negro si no hay estrella)
    pub fn sample(&self, _direction: &Vec3) -> Color {
        Color::new(0, 0, 0)
    }
}