// src/stars.rs
use nalgebra_glm::Vec3;
use crate::color::Color;

#[derive(Clone)]
pub struct Stars {
    pub density: f32,
}

impl Stars {
    pub fn new(density: f32) -> Self {
        Stars { density }
    }

    pub fn sample(&self, direction: &Vec3) -> Color {
        Color::new(0, 5, 15)
    }

}