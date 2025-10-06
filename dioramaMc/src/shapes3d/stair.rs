// src/stair.rs

use nalgebra_glm::Vec3;
use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;
use super::wall::Wall;

/// Escalera axis-aligned construida de 2 bloques AABB.
pub struct Stair {
    pub lower: Wall,
    pub upper: Wall,
}

impl Stair {
    pub fn from_center_edge(center: Vec3, edge: f32, material: Material, face_forward: bool) -> Self {
        let h = edge * 0.5;

        let lower_center = Vec3::new(center.x, center.y - h * 0.5, center.z);
        let lower = Wall::from_center_dims(lower_center, edge, h, edge, material.clone());

        let dz = h; // media profundidad
        let shift_z = if face_forward { dz * 0.5 } else { -dz * 0.5 };

        let upper_center = Vec3::new(center.x, center.y + h * 0.5, center.z + shift_z);
        let upper = Wall::from_center_dims(upper_center, edge, h, dz, material);

        Stair { lower, upper }
    }
}

impl RayIntersect for Stair {
    fn ray_intersect(&self, ro: &Vec3, rd: &Vec3) -> Intersect {
        let i1 = self.lower.ray_intersect(ro, rd);
        let i2 = self.upper.ray_intersect(ro, rd);

        match (i1.is_intersecting, i2.is_intersecting) {
            (false, false) => Intersect::empty(),
            (true,  false) => i1,
            (false, true ) => i2,
            (true,  true ) => if i1.distance <= i2.distance { i1 } else { i2 },
        }
    }
}