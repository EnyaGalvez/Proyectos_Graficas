// src/stair.rs
use nalgebra_glm::Vec3;
use std::sync::Arc;

use crate::intersect::{RayIntersect, Intersect};
use crate::material::Material;
use super::wall::Wall;

/// Escalera axis-aligned construida de 2 bloques AABB.
pub struct Stair {
    pub lower: Wall,
    pub upper: Wall,
}

#[derive(Debug, Clone, Copy)]
pub enum Facing {
    Forward,
    Backward,
    Right,
    Left
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Upright,
    UpsideDown
}

impl Stair {
    pub fn from_center_edge_control(center: Vec3, edge: f32, material: Arc<Material>, facing: Facing, orientation: Orientation) -> Self {
        let h = edge * 0.5;

        let vy = match orientation {
            Orientation::Upright => 1.0,
            Orientation::UpsideDown => -1.0
        };

        let lower_center = Vec3::new(center.x, center.y - vy * h * 0.5, center.z);
        let lower = Wall::from_center_dims(lower_center, edge, h, edge, material.clone());

        let upper = match facing {
            Facing::Forward => {
                let dz = h;
                let upper_center = Vec3::new(center.x, center.y + vy * h * 0.5, center.z + dz * 0.5);
                Wall::from_center_dims(upper_center, edge, h, dz, material.clone())
            }
            Facing::Backward => {
                let dz = h;
                let upper_center = Vec3::new(center.x, center.y + vy * h * 0.5, center.z - dz * 0.5);
                Wall::from_center_dims(upper_center, edge, h, dz, material.clone())
            }
            Facing::Right => {
                let dx = h;
                let upper_center = Vec3::new(center.x + dx * 0.5, center.y + vy * h * 0.5, center.z);
                Wall::from_center_dims(upper_center, dx, h, edge, material.clone())
            }
            Facing::Left => {
                let dx = h;
                let upper_center = Vec3::new(center.x - dx * 0.5, center.y + vy * h * 0.5, center.z);
                Wall::from_center_dims(upper_center, dx, h, edge, material.clone())
            }
        };

        Stair { lower, upper }
    }

    pub fn from_center_edge_oriented(center: Vec3, edge: f32, material: Arc<Material>, facing: Facing) -> Self {
        Self::from_center_edge_control(center, edge, material, facing, Orientation::Upright)
    }

    pub fn from_center_edge(center: Vec3, edge: f32, material: Arc<Material>, face_forward: bool) -> Self {
        let facing = if face_forward { Facing::Forward } else { Facing::Backward };
        Self::from_center_edge_oriented(center, edge, material, facing)
    }

    pub fn with_tiling(mut self, tu: f32, tv: f32) -> Self {
        self.lower = self.lower.with_tiling(tu, tv);
        self.upper = self.upper.with_tiling(tu, tv);
        self
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