// src/render.rs
use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::intersect::{Intersect, RayIntersect};
use crate::light::Light;
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

const SHADOW_BIAS: f32 = 1e-4;

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
    pub light: Light,
}

impl Scene {
    pub fn new(objects: Vec<Box<dyn RayIntersect>>, light: Light) -> Self {
        Self { objects, light }
    }
}

struct CameraParams {
    origin: Vec3,
    x_axis: Vec3,
    y_axis: Vec3,
    z_axis: Vec3,
    persp: f32,
    aspect: f32,
}

pub struct RenderPipeline;

impl RenderPipeline {
    pub fn new() -> Self { Self }

    fn build_camera_params(&self, fb: &Framebuffer, camera: &Camera) -> CameraParams {
        let (x_axis, y_axis, z_axis) = camera.axes();
        let w = fb.width as f32;
        let h = fb.height as f32;
        let aspect = w / h;
        let fov = PI / 3.0;
        let persp = (fov * 0.5).tan();

        CameraParams {
            origin: camera.eye,
            x_axis, y_axis, z_axis,
            persp, aspect,
        }
    }

    #[inline]
    fn reflect(&self, i: &Vec3, n: &Vec3) -> Vec3 {
        i - 2.0 * i.dot(n) * n
    }

    fn cast_shadow(&self, hit: &Intersect, light: &Light, objects: &[Box<dyn RayIntersect>]) -> f32 {
        let light_dir = (light.position - hit.point).normalize();
        let light_distance = (light.position - hit.point).magnitude();

        let offset = hit.normal * SHADOW_BIAS;
        let origin = if light_dir.dot(&hit.normal) < 0.0 {
            hit.point - offset
        } else {
            hit.point + offset
        };

        for obj in objects {
            let s = obj.ray_intersect(&origin, &light_dir);
            if s.is_intersecting && s.distance < light_distance {
                return 1.0;
            }
        }
        0.0
    }

    fn shade(&self, ray_o: &Vec3, ray_d: &Vec3, scene: &Scene,) -> Color {
        let mut best = Intersect::empty();
        let mut zbuf = f32::INFINITY;
        for obj in &scene.objects {
            let i = obj.ray_intersect(ray_o, ray_d);
            if i.is_intersecting && i.distance < zbuf {
                zbuf = i.distance;
                best = i;
            }
        }

        if !best.is_intersecting {
            return Color::new(0, 0, 26);
        }

        // Albedo (usando tiling U/V del material si hay UV)
        let base_color = if let Some(tex) = &best.material.albedo_map {
            if best.has_uv {
                let (u, v) = best.uv;
                tex.sample_tiled_mip(u, v, best.material.tiling_u, best.material.tiling_v)
            } else {
                best.material.diffuse
            }
        } else {
            best.material.diffuse
        };

        // Normal con normal map (TBN)
        let mut n = glm::normalize(&best.normal);
        if let (true, Some(nmap)) = (best.has_tangent, &best.material.normal_map) {
            if best.has_uv {
                let (u, v) = best.uv;
                let nt = nmap.sample_normal_tangent_mip_uv(u, v, best.material.tiling_u, best.material.tiling_v);
                let t = glm::normalize(&best.tangent);
                let b = glm::normalize(&best.bitangent);
                n = glm::normalize(&(n * 0.5 + (t * nt.x + b * nt.y + n * nt.z) * 0.5));
            }
        }

        // Iluminación
        let light_dir = (scene.light.position - best.point).normalize();
        let view_dir = (ray_o - best.point).normalize();
        let reflect_dir = self.reflect(&-light_dir, &n);

        // sombra
        let shadow = self.cast_shadow(&best, &scene.light, &scene.objects);
        let light_intensity = scene.light.intensity * (1.0 - shadow);

        let diff_i = n.dot(&light_dir).max(0.0).min(1.0);
        let diffuse = base_color * best.material.albedo[0] * diff_i * light_intensity;

        let spec_i = view_dir.dot(&reflect_dir).max(0.0).powf(best.material.specular);
        let specular = scene.light.color * best.material.albedo[1] * spec_i * light_intensity;

        diffuse + specular
    }

    pub fn render(&self, fb: &mut Framebuffer, scene: &Scene, camera: &Camera) {
        let params = self.build_camera_params(fb, camera);
        let persp  = params.persp;
        let aspect = params.aspect;
        let origin = params.origin;
        let (x_axis, y_axis, z_axis) = (params.x_axis, params.y_axis, params.z_axis);

        let width  = fb.width as usize;
        let height = fb.height as usize;
        
        for y in 0..height {
            let sy = -(2.0 * y as f32) / height as f32 + 1.0;
            let screen_y = sy * persp;

            let mut row = vec![0u32; width];
            for x in 0..width {
                let sx = (2.0 * x as f32) / width as f32 - 1.0;
                let screen_x = sx * aspect * persp;

                // Rayo en mundo
                let dir_world = x_axis * screen_x + y_axis * screen_y + z_axis;
                let dir_world = nalgebra_glm::normalize(&dir_world);

                let color = self.shade(&origin, &dir_world, scene);
                row[x] = color.to_hex();
            }
            fb.write_row(y, &row);
        }
    }
}
