// src/sphere.rs
use std::sync::Arc;
use nalgebra_glm::{Vec3, dot};
use std::f32::consts::PI;
use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<Material>) -> Self {
        Sphere { center, radius, material }
    }

    // Calcular coordenadas UV esféricas
    fn calculate_uv(&self, point: &Vec3) -> (f32, f32) {
        // Normalizar el punto respecto al centro de la esfera
        let d = (point - self.center).normalize();
        
        // Coordenadas esféricas
        // u = phi / 2π (ángulo horizontal)
        // v = theta / π (ángulo vertical)
        let u = 0.5 + (d.z.atan2(d.x)) / (2.0 * PI);
        let v = 0.5 - (d.y.asin()) / PI;
        
        (u, v)
    }

    // Calcular espacio tangente (TBN) para normal mapping
    fn calculate_tangent_space(&self, normal: &Vec3) -> (Vec3, Vec3) {
        // Elegir un vector arbitrario que no sea paralelo a la normal
        let up = if normal.y.abs() < 0.999 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        // Tangente = up × normal (perpendicular a ambos)
        let tangent = up.cross(normal).normalize();
        
        // Bitangente = normal × tangent
        let bitangent = normal.cross(&tangent).normalize();

        (tangent, bitangent)
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let oc = ray_origin - self.center;

        let a = dot(ray_direction, ray_direction);
        let b = 2.0 * dot(&oc, ray_direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant > 0.0 {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            
            if t > 0.0 {
                let point = ray_origin + ray_direction * t;
                
                let normal = (point - self.center).normalize();
                
                let distance = t;

                let (u, v) = self.calculate_uv(&point);

                let (tangent, bitangent) = self.calculate_tangent_space(&normal);

                return Intersect::new(point, normal, distance, self.material.clone())
                    .with_uv(u, v)
                    .with_tangent(tangent, bitangent);
            }
        }

        Intersect::empty()
    }
}