// src/cube.rs

use nalgebra_glm::Vec3;
use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3, // Esquina minima
    pub max: Vec3, // Esquina maxima
    pub material: Material,
}

impl Cube {
    /// Construye un cubo a partir de centro y lado
    pub fn from_center_size(center: Vec3, edge: f32, material: Material) -> Self {
        let h = edge * 0.5;
        Cube {
            min: Vec3::new(center.x - h, center.y - h, center.z - h),
            max: Vec3::new(center.x + h, center.y + h, center.z + h),
            material,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // slab method
        let mut tmin = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut tmax = (self.max.x - ray_origin.x) / ray_direction.x;
        if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

        let mut tymin = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut tymax = (self.max.y - ray_origin.y) / ray_direction.y;
        if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

        if tmin > tymax || tymin > tmax { return Intersect::empty(); }
        if tymin > tmin { tmin = tymin; }
        if tymax < tmax { tmax = tymax; }

        let mut tzmin = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut tzmax = (self.max.z - ray_origin.z) / ray_direction.z;
        if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

        if tmin > tzmax || tzmin > tmax { return Intersect::empty(); }
        if tzmin > tmin { tmin = tzmin; }
        if tzmax < tmax { tmax = tzmax; }

        // elegir la primera interseccion valida
        if tmin < 0.0 && tmax < 0.0 {
            return Intersect::empty();
        }
        let t = if tmin >= 0.0 { tmin } else { tmax };

        let point = ray_origin + ray_direction * t;

        // normal segun la cara tocada
        let eps = 1e-4;
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        let mut u = 0.0f32;
        let mut v = 0.0f32;

        let sx = self.max.x - self.min.x;
        let sy = self.max.y - self.min.y;
        let sz = self.max.z - self.min.z;

        // Mapas por cara (consistentes, origen abajo-izquierda en objeto)
        // Cara -X
        if (point.x - self.min.x).abs() < eps {
            normal = Vec3::new(-1.0, 0.0, 0.0);
            u = (point.z - self.min.z) / sz;
            v = (point.y - self.min.y) / sy;
        }
        // Cara +X
        else if (point.x - self.max.x).abs() < eps {
            normal = Vec3::new(1.0, 0.0, 0.0);
            u = (self.max.z - point.z) / sz;
            v = (point.y - self.min.y) / sy;
        }
        // Cara -Y
        else if (point.y - self.min.y).abs() < eps {
            normal = Vec3::new(0.0, -1.0, 0.0);
            u = (point.x - self.min.x) / sx;
            v = (self.max.z - point.z) / sz;
        }
        // Cara +Y
        else if (point.y - self.max.y).abs() < eps {
            normal = Vec3::new(0.0, 1.0, 0.0);
            u = (point.x - self.min.x) / sx;
            v = (point.z - self.min.z) / sz;
        }
        // Cara -Z
        else if (point.z - self.min.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, -1.0);
            u = (self.max.x - point.x) / sx;
            v = (point.y - self.min.y) / sy;
        }
        // Cara +Z
        else if (point.z - self.max.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, 1.0);
            u = (point.x - self.min.x) / sx;
            v = (point.y - self.min.y) / sy;
        }
    Intersect::new(point, normal, t, self.material.clone()).with_uv(u, v)
    }
}