// src/aabb.rs
use nalgebra_glm::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
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

    // Test AABB vs rayo, acotado por t_max (si ya tienes un hit más cercano).
    #[inline]
    pub fn hit_ray(&self, ro: &Vec3, rd: &Vec3, t_max: f32) -> bool {
        // Evita ambigüedad: tipa todo como f32 explícitamente
        let invx: f32 = 1.0 / rd.x;
        let invy: f32 = 1.0 / rd.y;
        let invz: f32 = 1.0 / rd.z;

        let mut tmin: f32 = 0.0;
        let mut tmax_local: f32 = t_max;

        // X
        let mut t0: f32 = (self.min.x - ro.x) * invx;
        let mut t1: f32 = (self.max.x - ro.x) * invx;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax_local = tmax_local.min(t1);
        if tmax_local < tmin { return false; }

        // Y
        t0 = (self.min.y - ro.y) * invy;
        t1 = (self.max.y - ro.y) * invy;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax_local = tmax_local.min(t1);
        if tmax_local < tmin { return false; }

        // Z
        t0 = (self.min.z - ro.z) * invz;
        t1 = (self.max.z - ro.z) * invz;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax_local = tmax_local.min(t1);
        if tmax_local < tmin { return false; }

        // Acepta hits delante de la cámara
        tmax_local > 0.0
    }
}
