// src/light.rs

use nalgebra_glm::Vec3;
use crate::color::Color;

#[derive(Clone)]
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub casts_shadows: bool
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            casts_shadows: true
        }
    }

    pub fn no_shadow(mut self) -> Self {
        self.casts_shadows = false;
        self
    }
}