// src/wall.rs

use nalgebra_glm::Vec3;
use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;

/// Prisma rectangular alineado a ejes.
pub struct Wall {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
    pub uv_tu: f32,
    pub uv_tv: f32,
}

impl Wall {
    pub fn from_center_dims(center: Vec3, sx: f32, sy: f32, sz: f32, material: Material) -> Self {
        let hx = sx * 0.5;
        let hy = sy * 0.5;
        let hz = sz * 0.5;
        Wall {
            min: Vec3::new(center.x - hx, center.y - hy, center.z - hz),
            max: Vec3::new(center.x + hx, center.y + hy, center.z + hz),
            material,
            uv_tu: 1.0,
            uv_tv: 1.0,
        }
    }

    pub fn with_tiling(mut self, tu: f32, tv: f32) -> Self {
    self.uv_tu = tu;
    self.uv_tv = tv;
    self
}
}

impl RayIntersect for Wall {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
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

        if tmin < 0.0 && tmax < 0.0 {
            return Intersect::empty();
        }
        let t = if tmin >= 0.0 { tmin } else { tmax };
        let point = ray_origin + ray_direction * t;

        let eps = 1e-4;
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        let mut u = 0.0f32;
        let mut v = 0.0f32;

        let mut tangent = Vec3::new(1.0, 0.0, 0.0);
        let mut bitangent = Vec3::new(0.0, 1.0, 0.0);

        let sx = self.max.x - self.min.x;
        let sy = self.max.y - self.min.y;
        let sz = self.max.z - self.min.z;

        if (point.x - self.min.x).abs() < eps {
            normal = Vec3::new(-1.0, 0.0, 0.0);
            u = ((point.z - self.min.z) / sz) * self.material.tiling_u;
            v = ((point.y - self.min.y) / sy) * self.material.tiling_v;
            tangent = Vec3::new(0.0, 0.0, 1.0);
            bitangent = Vec3::new(0.0, 1.0, 0.0);
        } 
        else if (point.x - self.max.x).abs() < eps {
            normal = Vec3::new(1.0, 0.0, 0.0);
            u = ((self.max.z - point.z) / sz) * self.material.tiling_u;
            v = ((point.y - self.min.y) / sy) * self.material.tiling_v;
            tangent = Vec3::new(0.0, 0.0, -1.0);
            bitangent = Vec3::new(0.0, 1.0, 0.0);
        } 
        else if (point.y - self.min.y).abs() < eps {
            normal = Vec3::new(0.0, -1.0, 0.0);
            u = ((point.x - self.min.x) / sx) * self.material.tiling_u;
            v = ((self.max.z - point.z) / sz) * self.material.tiling_v;
            tangent = Vec3::new(1.0, 0.0, 0.0);
            bitangent = Vec3::new(0.0, 0.0, -1.0);
        } 
        else if (point.y - self.max.y).abs() < eps {
            normal = Vec3::new(0.0, 1.0, 0.0);
            u = ((point.x - self.min.x) / sx) * self.material.tiling_u;
            v = ((point.z - self.min.z) / sz) * self.material.tiling_v;
            tangent = Vec3::new(1.0, 0.0, 0.0);
            bitangent = Vec3::new(0.0, 0.0, 1.0);
        } 
        else if (point.z - self.min.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, -1.0);
            u = ((self.max.x - point.x) / sx) * self.material.tiling_u;
            v = ((point.y - self.min.y) / sy) * self.material.tiling_v;
            tangent = Vec3::new(-1.0, 0.0, 0.0);
            bitangent = Vec3::new(0.0, 1.0, 0.0);
        } 
        else if (point.z - self.max.z).abs() < eps {
            normal = Vec3::new(0.0, 0.0, 1.0);
            u = ((point.x - self.min.x) / sx) * self.material.tiling_u;
            v = ((point.y - self.min.y) / sy) * self.material.tiling_v;
            tangent = Vec3::new(1.0, 0.0, 0.0);
            bitangent = Vec3::new(0.0, 1.0, 0.0);
        }

        u *= self.uv_tu;
        v *= self.uv_tv;

        Intersect::new(point, normal, t, self.material.clone()).with_uv(u, v).with_tangent(tangent, bitangent)
    }
}