// srrc/ring.rs
use std::sync::Arc;
use nalgebra_glm::Vec3;
use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;

#[derive(Clone)]
pub struct Ring {
    pub center: Vec3,
    pub normal: Vec3,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub material: Arc<Material>,
}

impl Ring {
    pub fn new(center: Vec3, normal: Vec3, inner: f32, outer: f32, material: Arc<Material>) -> Self {
        Ring {
            center,
            normal: normal.normalize(),
            inner_radius: inner,
            outer_radius: outer,
            material,
        }
    }
}

impl RayIntersect for Ring {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let denom = ray_direction.dot(&self.normal);
        
        // Paralelo al plano?
        if denom.abs() < 1e-6 {
            return Intersect::empty();
        }
        
        let t = (self.center - ray_origin).dot(&self.normal) / denom;
        
        if t < 0.0 {
            return Intersect::empty();
        }
        
        let point = ray_origin + ray_direction * t;
        let to_point = point - self.center;
        let distance_from_center = to_point.magnitude();
        
        // Verificar si está dentro del anillo
        if distance_from_center >= self.inner_radius && 
           distance_from_center <= self.outer_radius {
            
            // UV mapping circular
            let angle = to_point.z.atan2(to_point.x);
            let u = (angle / (2.0 * std::f32::consts::PI) + 0.5).fract();
            let v = (distance_from_center - self.inner_radius) / 
                    (self.outer_radius - self.inner_radius);
            
            return Intersect::new(point, self.normal, t, self.material.clone())
                .with_uv(u, v);
        }
        
        Intersect::empty()
    }
}