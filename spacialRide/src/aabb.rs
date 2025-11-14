// src/aabb.rs
use nalgebra_glm::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_sphere(center: Vec3, radius: f32) -> Self {
        let r = Vec3::new(radius, radius, radius);
        Self { min: center - r, max: center + r }
    }

    #[inline]
    pub fn union(a: AABB, b: AABB) -> AABB {
        AABB {
            min: Vec3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            max: Vec3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        }
    }

    // Ray–AABB robusto (slabs + epsilon) acotado por max_dist
    #[inline]
    pub fn hit_ray(&self, ro: &Vec3, rd: &Vec3, max_dist: f32) -> bool {
        const EPS: f32 = 1e-8;

        // X
        let (mut tmin, mut tmax) = if rd.x.abs() < EPS {
            if ro.x < self.min.x || ro.x > self.max.x { return false; }
            (f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let inv = 1.0 / rd.x;
            let t0 = (self.min.x - ro.x) * inv;
            let t1 = (self.max.x - ro.x) * inv;
            (t0.min(t1), t0.max(t1))
        };

        // Y
        let (tymin, tymax) = if rd.y.abs() < EPS {
            if ro.y < self.min.y || ro.y > self.max.y { return false; }
            (f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let inv = 1.0 / rd.y;
            let t0 = (self.min.y - ro.y) * inv;
            let t1 = (self.max.y - ro.y) * inv;
            (t0.min(t1), t0.max(t1))
        };
        tmin = tmin.max(tymin);
        tmax = tmax.min(tymax);
        if tmax < tmin { return false; }

        // Z
        let (tzmin, tzmax) = if rd.z.abs() < EPS {
            if ro.z < self.min.z || ro.z > self.max.z { return false; }
            (f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let inv = 1.0 / rd.z;
            let t0 = (self.min.z - ro.z) * inv;
            let t1 = (self.max.z - ro.z) * inv;
            (t0.min(t1), t0.max(t1))
        };
        tmin = tmin.max(tzmin);
        tmax = tmax.min(tzmax);
        if tmax < tmin { return false; }

        // delante de la cámara y dentro del rango
        if tmax < 0.0 { return false; }
        tmin < max_dist
    }
}
