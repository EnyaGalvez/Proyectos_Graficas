// material.rs
use crate::color::Color;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  pub albedo: [f32; 2],
  pub texture: Option<std::sync::Arc<Texture>>,
}

impl Material {
  pub fn new(
    diffuse: Color,
    specular: f32,
    albedo: [f32; 2]
  ) -> Self {
    Material {
      diffuse,
      specular,
      albedo,
      texture: None,
    }
  }

  pub fn with_texture(
    diffuse: Color, 
    specular: f32, 
    albedo: [f32; 2], 
    tex: std::sync::Arc<Texture>
  ) -> Self {
    Material { 
      diffuse, 
      specular, 
      albedo, 
      texture: Some(tex),
    }
  }


  pub fn black() -> Self {
    Material {
      diffuse: Color::new(1, 0, 0),
      specular: 2.0,
      albedo: [0.0, 1.0],
      texture: None,
    }
  }
}