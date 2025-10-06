// src/texture.rs
use crate::color::Color;

#[derive(Debug)]
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

    pub fn sample(&self, u: f32, v: f32) -> Color {
        // wrap (tiling)
        let u = u.fract().rem_euclid(1.0);
        let v = v.fract().rem_euclid(1.0);

        let x = (u * (self.w as f32 - 1.0)).round() as usize;
        let y = ((1.0 - v) * (self.h as f32 - 1.0)).round() as usize; // origen arriba-izquierda
        self.data[y * self.w + x]
    }
}