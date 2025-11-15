// src/controller.rs
use minifb::{Key, Window};
use crate::camera::Camera;

pub struct Controller {
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32
}

impl Controller {
    pub fn new(move_speed: f32, rotate_speed: f32, zoom_speed: f32) -> Self {
        Self { move_speed, rotate_speed, zoom_speed }
    }

    pub fn update(&mut self, window: &Window, camera: &mut Camera) {
        // Movimiento hacia adelante/atrás (W/S)
        if window.is_key_down(Key::W) {
            camera.move_forward(self.move_speed);
        }

        if window.is_key_down(Key::S) {
            camera.move_forward(-self.move_speed);
        }

        // Movimiento lateral y vertical (A/D/Space/LeftShift)

        if window.is_key_down(Key::A) {
            camera.pan(-self.move_speed, 0.0);
        }

        if window.is_key_down(Key::D) {
            camera.pan(self.move_speed, 0.0);
        }

        // Rotación con flechas
        if window.is_key_down(Key::Left) {
            camera.orbit(self.rotate_speed, 0.0);
        }

        if window.is_key_down(Key::Right) {
            camera.orbit(-self.rotate_speed, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, self.rotate_speed);
        }

        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, -self.rotate_speed);
        }

        // Zoom
        if window.is_key_down(Key::Q) {
            camera.dolly(self.zoom_speed);
        }

        if window.is_key_down(Key::E) {
            camera.dolly(-self.zoom_speed);
        }
    }
}