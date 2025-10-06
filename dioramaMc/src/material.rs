// material.rs
use crate::color::Color;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  pub albedo: [f32; 2],
  pub albedo_map: Option<Texture>,
  pub normal_map: Option<Texture>,
  pub tiling: f32
}

impl Material {
  pub fn new( diffuse: Color, specular: f32, albedo: [f32; 2]) -> Self {
    Self { diffuse, specular, albedo, albedo_map: None, normal_map: None, tiling: 1.0 }
  }

  pub fn with_albedo_map(mut self, tex: Texture, tiling: f32) -> Self {
        self.albedo_map = Some(tex);
        self.tiling = tiling;
        self
    }

  pub fn with_normal_map(mut self, tex: Texture) -> Self {
        self.normal_map = Some(tex);
        self
    }

  pub fn black() -> Self {
    Material {
      diffuse: Color::new(0, 0, 0),
      specular: 4.0,
      albedo: [0.0, 0.2],
      albedo_map: None,
      normal_map: None,
      tiling: 1.0,
    }
  }
}