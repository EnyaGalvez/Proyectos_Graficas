// src/model_renderer.rs
use nalgebra_glm::{Vec3};
use crate::obj_loader::{load_obj, setup_vertex_array, Face};
use crate::vertex_shader::{VertexShader, perspective, look_at, model_matrix};
use crate::rasterizer::{draw_triangle, draw_triangle_wireframe, ZBuffer};
use crate::framebuffer::Framebuffer;
use crate::color::Color;
use crate::camera::Camera;
use std::f32::consts::PI;

pub struct Model3D {
    vertices: Vec<Vec3>,
    faces: Vec<Face>,
    vertex_array: Vec<Vec3>,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub color: Color,
}

impl Model3D {
    /// Carga un modelo desde un archivo OBJ
    pub fn from_obj(path: &str) -> Result<Self, String> {
        let (vertices, faces) = load_obj(path)?;
        let vertex_array = setup_vertex_array(&vertices, &faces);
        
        Ok(Self {
            vertices,
            faces,
            vertex_array,
            position: Vec3::zeros(),
            rotation: Vec3::zeros(),
            scale: 1.0,
            color: Color::new(200, 200, 200),
        })
    }
    
    /// Crea un cubo simple (para testing)
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vec3::new(-0.5, -0.5,  0.5),
            Vec3::new( 0.5, -0.5,  0.5),
            Vec3::new( 0.5,  0.5,  0.5),
            Vec3::new(-0.5,  0.5,  0.5),
            // Back face
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new( 0.5, -0.5, -0.5),
            Vec3::new( 0.5,  0.5, -0.5),
            Vec3::new(-0.5,  0.5, -0.5),
        ];
        
        let faces = vec![
            Face { vertex_indices: vec![[0, 1, 2], [0, 2, 3]] }, // Front
            Face { vertex_indices: vec![[5, 4, 7], [5, 7, 6]] }, // Back
            Face { vertex_indices: vec![[4, 0, 3], [4, 3, 7]] }, // Left
            Face { vertex_indices: vec![[1, 5, 6], [1, 6, 2]] }, // Right
            Face { vertex_indices: vec![[3, 2, 6], [3, 6, 7]] }, // Top
            Face { vertex_indices: vec![[4, 5, 1], [4, 1, 0]] }, // Bottom
        ];
        
        let vertex_array = setup_vertex_array(&vertices, &faces);
        
        Self {
            vertices,
            faces,
            vertex_array,
            position: Vec3::zeros(),
            rotation: Vec3::zeros(),
            scale: 1.0,
            color: Color::new(100, 200, 255),
        }
    }
    
    /// Renderiza el modelo
    pub fn render(
        &self,
        framebuffer: &mut Framebuffer,
        zbuffer: &mut ZBuffer,
        camera: &Camera,
        width: usize,
        height: usize,
        wireframe: bool,
    ) {
        // Crear matrices de transformación
        let aspect = width as f32 / height as f32;
        let fov = PI / 3.0; // 60 grados
        
        let proj = perspective(fov, aspect, 0.1, 100.0);
        let view = look_at(camera.eye, camera.center, camera.up);
        let model = model_matrix(self.position, self.rotation, self.scale);
        
        let shader = VertexShader::new(proj, view, model);
        
        // Renderizar cada triángulo
        for i in (0..self.vertex_array.len()).step_by(3) {
            let v0 = self.vertex_array[i];
            let v1 = self.vertex_array[i + 1];
            let v2 = self.vertex_array[i + 2];
            
            // Proyectar vértices a pantalla
            let s0 = shader.project_screen(v0, width, height);
            let s1 = shader.project_screen(v1, width, height);
            let s2 = shader.project_screen(v2, width, height);
            
            // Culling: no dibujar triángulos fuera de pantalla o detrás de la cámara
            if s0.z < 0.0 || s1.z < 0.0 || s2.z < 0.0 {
                continue;
            }
            
            // Backface culling
            let edge1 = Vec3::new(s1.x - s0.x, s1.y - s0.y, 0.0);
            let edge2 = Vec3::new(s2.x - s0.x, s2.y - s0.y, 0.0);
            let cross = edge1.x * edge2.y - edge1.y * edge2.x;
            
            if cross < 0.0 {
                continue; // Triángulo mirando hacia atrás
            }
            
            if wireframe {
                draw_triangle_wireframe(framebuffer, s0, s1, s2, self.color);
            } else {
                draw_triangle(framebuffer, zbuffer, s0, s1, s2, self.color);
            }
        }
    }
}

pub struct ModelRenderer {
    zbuffer: ZBuffer,
}

impl ModelRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            zbuffer: ZBuffer::new(width, height),
        }
    }
    
    pub fn clear_zbuffer(&mut self) {
        self.zbuffer.clear();
    }
    
    pub fn render_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &Model3D,
        camera: &Camera,
        wireframe: bool,
    ) {
        model.render(
            framebuffer,
            &mut self.zbuffer,
            camera,
            framebuffer.width,
            framebuffer.height,
            wireframe,
        );
    }
}