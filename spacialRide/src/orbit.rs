// src/orbit.rs
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Orbit {
    pub center: Vec3,
    pub radius: f32,
    pub speed: f32,
    pub tilt: f32,
    pub phase: f32,
}

impl Orbit {
    pub fn new(center: Vec3, radius: f32, speed: f32) -> Self {
        Orbit {
            center,
            radius,
            speed,
            tilt: 0.0,
            phase: 0.0,
        }
    }
    
    pub fn with_tilt(mut self, tilt_degrees: f32) -> Self {
        self.tilt = tilt_degrees * PI / 180.0;
        self
    }
    
    pub fn with_phase(mut self, phase_degrees: f32) -> Self {
        self.phase = phase_degrees * PI / 180.0;
        self
    }
    
    pub fn position_at(&self, time: f32) -> Vec3 {
        let angle = time * self.speed + self.phase;
        
        // Posición en plano XZ
        let x = self.radius * angle.cos();
        let z = self.radius * angle.sin();
        
        // Aplicar inclinación
        let y = z * self.tilt.sin();
        let z_tilted = z * self.tilt.cos();
        
        self.center + Vec3::new(x, y, z_tilted)
    }
}