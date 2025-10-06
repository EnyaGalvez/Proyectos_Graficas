// src/texture.rs
use image::io::Reader as ImageReader;
use nalgebra_glm::Vec3;
use crate::color::Color;

#[derive(Debug, Clone)]
pub struct Texture {
    pub w: usize,
    pub h: usize,
    pub data: Vec<Color>,
}

impl Texture {
    pub fn from_rgb8(w: usize, h: usize, bytes: &[u8]) -> Self {
        assert!(bytes.len() == w * h * 3);
        let mut data = Vec::with_capacity(w * h);
        for i in (0..bytes.len()).step_by(3) {
            data.push(Color::new(bytes[i], bytes[i + 1], bytes[i + 2]));
        }
        Texture { w, h, data }
    }

    pub fn from_file(path: &str) -> Self {
        let img = match ImageReader::open(path) {
            Ok(reader) => reader.decode().expect("Error al decodificar imagen"),
            Err(_) => panic!("No se pudo abrir la textura en la ruta: {}", path),
        };

        let img = img.to_rgb8();
        let (w, h) = img.dimensions();
        let mut data = Vec::with_capacity((w * h) as usize);

        for pixel in img.pixels() {
            data.push(Color::new(pixel[0], pixel[1], pixel[2]));
        }

        Texture { w: w as usize, h: h as usize, data }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        // wrap (tiling)
        let u = u.fract().rem_euclid(1.0);
        let v = v.fract().rem_euclid(1.0);

        let x = (u * (self.w as f32 - 1.0)).round() as usize;
        let y = ((1.0 - v) * (self.h as f32 - 1.0)).round() as usize; // origen arriba-izquierda
        self.data[y * self.w + x]
    }

    pub fn sample_tiled(&self, u: f32, v: f32, tiling: f32) -> Color {
        let u = (u * tiling).fract().rem_euclid(1.0);
        let v = (v * tiling).fract().rem_euclid(1.0);
        let x = (u * (self.w as f32 - 1.0)).round() as usize;
        let y = ((1.0 - v) * (self.h as f32 - 1.0)).round() as usize;
        self.data[y * self.w + x]
    }

    pub fn sample_normal_tangent(&self, u: f32, v: f32, tiling: f32) -> Vec3 {
        let c = self.sample_tiled(u, v, tiling);
        let nx = (c.r as f32 / 255.0) * 2.0 - 1.0;
        let ny = (c.g as f32 / 255.0) * 2.0 - 1.0;
        let nz = (c.b as f32 / 255.0) * 2.0 - 1.0;
        nalgebra_glm::normalize(&Vec3::new(nx, ny, nz))
    }
}