// src/vertex_shader.rs
use nalgebra_glm::{Mat4, Vec3, Vec4};

pub struct VertexShader {
    pub proj: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

impl VertexShader {
    pub fn new(proj: Mat4, view: Mat4, model: Mat4) -> Self {
        Self { proj, view, model }
    }
    
    /// Proyecta un punto del espacio mundial a NDC (Normalized Device Coordinates)
    pub fn project_ndc(&self, p_world: Vec3) -> Vec3 {
        let p4 = self.proj * self.view * self.model * Vec4::new(p_world.x, p_world.y, p_world.z, 1.0);
        
        let w = if p4.w != 0.0 { p4.w } else { 1.0 };
        
        Vec3::new(p4.x / w, p4.y / w, p4.z / w)
    }
    
    /// Convierte de NDC a coordenadas de pantalla
    pub fn ndc_to_screen(&self, p_ndc: Vec3, width: usize, height: usize) -> Vec3 {
        let x = ((p_ndc.x + 1.0) * 0.5) * (width as f32);
        let y = ((1.0 - (p_ndc.y + 1.0) * 0.5)) * (height as f32); // Y invertido
        
        Vec3::new(x, y, p_ndc.z)
    }
    
    /// Proyecta directamente a coordenadas de pantalla
    pub fn project_screen(&self, p_world: Vec3, width: usize, height: usize) -> Vec3 {
        let ndc = self.project_ndc(p_world);
        self.ndc_to_screen(ndc, width, height)
    }
}

/// Crea una matriz de proyección en perspectiva
pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    nalgebra_glm::perspective(aspect, fov, near, far)
}

/// Crea una matriz de vista (look-at)
pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    nalgebra_glm::look_at(&eye, &center, &up)
}

/// Crea una matriz de modelo (transformaciones del objeto)
pub fn model_matrix(translation: Vec3, rotation: Vec3, scale: f32) -> Mat4 {
    let mut m = Mat4::identity();
    
    // Escala
    m = nalgebra_glm::scale(&m, &Vec3::new(scale, scale, scale));
    
    // Rotaciones (XYZ)
    m = nalgebra_glm::rotate(&m, rotation.x, &Vec3::new(1.0, 0.0, 0.0));
    m = nalgebra_glm::rotate(&m, rotation.y, &Vec3::new(0.0, 1.0, 0.0));
    m = nalgebra_glm::rotate(&m, rotation.z, &Vec3::new(0.0, 0.0, 1.0));
    
    // Traslación
    m = nalgebra_glm::translate(&m, &translation);
    
    m
}