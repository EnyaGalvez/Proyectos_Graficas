// src/intersect.rs
use nalgebra_glm::Vec3;
use crate::material::Material;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: Material,
    pub uv: (f32, f32),
    pub has_uv: bool,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub has_tangent: bool
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
            uv: (0.0, 0.0),
            has_uv: false,
            tangent: Vec3::zeros(),
            bitangent: Vec3::zeros(),
            has_tangent: false
        }
    }

    pub fn with_uv(mut self, u: f32, v: f32) -> Self { // helper
        self.uv = (u, v);
        self.has_uv = true;
        self
    }

    pub fn with_tangent(mut self, t: Vec3, b: Vec3) -> Self {
        self.tangent = t;
        self.bitangent = b;
        self.has_tangent = true;
        self
    }

    pub fn empty() -> Self {
        Intersect {
            point: Vec3::zeros(),
            normal: Vec3::zeros(),
            distance: 10.0,
            is_intersecting: false,
            material: Material::black(),
            uv: (0.0, 0.0),
            has_uv: false,
            tangent: Vec3::zeros(),
            bitangent: Vec3::zeros(),
            has_tangent: false
        }
    }
}

pub trait RayIntersect: Send + Sync {
  fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}