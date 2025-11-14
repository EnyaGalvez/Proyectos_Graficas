// material.rs
use std::sync::Arc;
use crate::color::Color;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  pub albedo: [f32; 2],
  // Texturas
  pub albedo_map: Option<Arc<Texture>>,
  pub normal_map: Option<Arc<Texture>>,
  // Tiling
  pub albedo_tu: f32,
  pub albedo_tv: f32,
  pub normal_tu: f32,
  pub normal_tv: f32,
  // Reflectividad, transparencia e indice de refraccion
  pub kr: f32,
  pub kt: f32,
  pub ior: f32,
  pub casts_shadows: bool
}

impl Material {
  pub fn new( diffuse: Color, specular: f32, albedo: [f32; 2],  kr: f32, kt: f32, ior: f32) -> Self {
    Material {
      diffuse,
      specular,
      albedo,
      kr,
      kt,
      ior,
      albedo_map: None,
      normal_map: None,
      albedo_tu: 1.0,
      albedo_tv: 1.0,
      normal_tu: 1.0,
      normal_tv: 1.0,
      casts_shadows: true,
    }
  }
  
  pub fn black() -> Self {
    Material::new(
        Color::new(0, 0, 0),
        0.0,
        [0.0, 0.0],
        0.0,
        0.0,
        1.0,
    )
  }

  pub fn no_shadow(mut self) -> Self {
        self.casts_shadows = false;
        self
    }

  pub fn with_albedo_map(mut self, texture: Arc<Texture>) -> Self {
      self.albedo_map = Some(texture);
      self
  }

  pub fn with_normal_map(mut self, texture: Arc<Texture>) -> Self {
      self.normal_map = Some(texture);
      self
  }

  pub fn with_albedo_tiling(mut self, tu: f32, tv: f32) -> Self {
      self.albedo_tu = tu;
      self.albedo_tv = tv;
      self
  }

  pub fn with_normal_tiling(mut self, tu: f32, tv: f32) -> Self {
      self.normal_tu = tu;
      self.normal_tv = tv;
      self
  }
}