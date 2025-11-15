// src/controller.rs
use minifb::{Key, Window};
use crate::camera::{Camera, CameraViewMode};

pub struct Controller {
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    last_view_mode_toggle: std::time::Instant,
}

impl Controller {
    pub fn new(move_speed: f32, rotate_speed: f32, zoom_speed: f32) -> Self {
        Self { move_speed, rotate_speed, zoom_speed, last_view_mode_toggle: std::time::Instant::now() }
    }

    pub fn update(&mut self, window: &Window, camera: &mut Camera) {
        // Modo de vista (V)
        if window.is_key_down(Key::V) {
            let now = std::time::Instant::now();
            if now.duration_since(self.last_view_mode_toggle).as_millis() > 200 {
                camera.toggle_view_mode();
                self.last_view_mode_toggle = now;
                
                // Imprimir modo actual
                match camera.view_mode {
                    CameraViewMode::FirstPerson => println!("Modo: Primera Persona"),
                    CameraViewMode::ThirdPerson => println!("Modo: Tercera Persona (con nave)"),
                }
            }
        }

        
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

        if window.is_key_down(Key::LeftShift) {
            camera.pan(0.0, -self.move_speed);
        }
        if window.is_key_down(Key::Space) {
            camera.pan(0.0, self.move_speed);
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