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
  pub tiling_u: f32,
  pub tiling_v: f32,
  pub kr: f32,
  pub kt: f32,
  pub ior: f32
}

impl Material {
  pub fn with_reflectance(mut self, kr: f32) -> Self {
      self.kr = kr.clamp(0.0, 1.0);
      self
  }

  pub fn with_transparency(mut self, kt: f32, ior: f32) -> Self {
      self.kt = kt.clamp(0.0, 1.0);
      self.ior = ior.max(1.0);
      self
  }

  pub fn new( diffuse: Color, specular: f32, albedo: [f32; 2]) -> Self {
    Self { 
      diffuse, 
      specular, 
      albedo, 
      albedo_map: None, 
      normal_map: None, 
      tiling_u: 1.0, 
      tiling_v: 1.0, 
      kr: 0.0,
      kt: 0.0,
      ior: 1.0 
    }
  }

  pub fn with_albedo_map(mut self, tex: Texture, tiling_u: f32, tiling_v: f32) -> Self {
        self.albedo_map = Some(tex);
        self.tiling_u = tiling_u;
        self.tiling_v = tiling_v;
        self
    }

  pub fn with_normal_map(mut self, tex: Texture, tu: f32, tv: f32) -> Self {
        self.normal_map = Some(tex);
        self.tiling_u = tu;
        self.tiling_v = tv;
        self
    }

  pub fn black() -> Self {
    Material {
      diffuse: Color::new(0, 0, 0),
      specular: 4.0,
      albedo: [0.0, 0.2],
      albedo_map: None,
      normal_map: None,
      tiling_u: 1.0,
      tiling_v: 1.0,
      kr: 0.0,
      kt: 0.0,
      ior: 1.0
    }
  }
}