// src/render.rs
use rayon::prelude::*;
use std::sync::Arc;
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::intersect::{Intersect, RayIntersect};
use crate::light::Light;

const SHADOW_BIAS: f32 = 1e-4;
const MAX_DEPTH: u32 = 3;

pub struct Scene {
    pub objects: Vec<Arc<dyn RayIntersect>>,
    pub light: Light,
}

impl Scene {
    pub fn new(objects: Vec<Arc<dyn RayIntersect>>, light: Light) -> Self {
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

    #[inline]
    fn fresnel_schlick(&self, cos_theta: f32, f0: f32) -> f32 {
        f0 + (1.0 - f0) * (1.0 - cos_theta).powf(5.0)
    }

    #[inline]
    fn refract(&self, i: &Vec3, n: &Vec3, eta: f32) -> Option<Vec3> {
        let cosi = (-*i).dot(n).clamp(-1.0, 1.0);
        let sin2_t = eta * eta * (1.0 - cosi * cosi);
        if sin2_t > 1.0 {
            return None;
        }
        let cost = (1.0 - sin2_t).sqrt();
        Some(eta * *i + (eta * cosi - cost) * *n)
    }

    fn cast_shadow(&self, hit: &Intersect, light: &Light, objects: &[Arc<dyn RayIntersect>]) -> f32 {
        let light_dir = (light.position - hit.point).normalize();
        let light_distance = (light.position - hit.point).magnitude();

        let offset = hit.normal * SHADOW_BIAS;
        let mut origin = if light_dir.dot(&hit.normal) < 0.0 {
            hit.point - offset
        } else {
            hit.point + offset
        };

        let mut transmittance = 1.0_f32;
        let mut traveled = 0.0_f32;

        loop {
            let mut nearest: Option<Intersect> = None;
            let mut min_d = light_distance - traveled;

            for obj in objects {
                let s = obj.ray_intersect(&origin, &light_dir);
                if s.is_intersecting && s.distance < min_d {
                    min_d = s.distance;
                    nearest = Some(s);
                }
            }
            match nearest { 
                Some(s) => { 
                    let m = &s.material;

                    if m.kt > 0.0 {
                        
                        let n_use = s.normal.normalize();
                        let cosi = light_dir.dot(&n_use).abs();
                        let f0 = ((m.ior - 1.0) / (m.ior + 1.0)).powi(2);
                        let kf = self.fresnel_schlick(cosi, f0);
                        let t = (m.kt * (1.0 - kf)).clamp(0.0, 1.0);

                        transmittance *= t;

                        // avanzar dentro del objeto
                        let step = min_d + SHADOW_BIAS;
                        traveled += step;
                        origin += light_dir * step;

                        if transmittance < 0.01 || traveled >= light_distance {
                            break;
                        }
                    } else {
                        return 1.0;
                    }
                }
                None => break,
            }
        }
        1.0 - transmittance  
    }

    fn shade(&self, ray_o: &Vec3, ray_d: &Vec3, scene: &Scene, depth: u32) -> Color {
        if depth >= MAX_DEPTH {
            return Color::new(0, 0, 26);
        }

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

        let n_geom = glm::normalize(&best.normal);
        let light_dir = (scene.light.position - best.point).normalize();
        let ndotl_geom = n_geom.dot(&light_dir);

        let mut n = n_geom;
        if ndotl_geom > 0.0 {
            if let (true, Some(nmap)) = (best.has_tangent, &best.material.normal_map) {
                if best.has_uv {
                    let (u, v) = best.uv;
                    let nt = nmap.sample_normal_tangent_mip_uv(u, v, best.material.normal_tu, best.material.normal_tv);
                    let t = glm::normalize(&best.tangent);
                    let b = glm::normalize(&best.bitangent);
                    n = glm::normalize(&(t * nt.x + b * nt.y + n * nt.z));
                }
            }
        }

        // Albedo (usando tiling U/V del material si hay UV)
        let base_color = if let Some(tex) = &best.material.albedo_map {
            if best.has_uv {
                let (u, v) = best.uv;
                tex.sample_tiled_mip(u, v, best.material.albedo_tu, best.material.albedo_tv)
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
                let nt = nmap.sample_normal_tangent_mip_uv(u, v, best.material.normal_tu, best.material.normal_tv);                let t = glm::normalize(&best.tangent);
                let b = glm::normalize(&best.bitangent);
                n = glm::normalize(&(n * 0.5 + (t * nt.x + b * nt.y + n * nt.z) * 0.5));
            }
        }

        // Iluminación
        let view_dir = (ray_o - best.point).normalize();
        let reflect_dir = self.reflect(&-light_dir, &n);
        let (diffuse, specular, shadow) = if ndotl_geom <= 0.0 {
            (Color::new(0,0,0), Color::new(0,0,0), 0.0)
        } else {
            let shadow = self.cast_shadow(&best, &scene.light, &scene.objects);
            let light_intensity = scene.light.intensity * (1.0 - shadow);

            let diff_i = n.dot(&light_dir).max(0.0);
            let diffuse = base_color * best.material.albedo[0] * diff_i * light_intensity;

            let reflect_dir = self.reflect(&-light_dir, &n);
            let spec_i = view_dir.dot(&reflect_dir).max(0.0).powf(best.material.specular);
            let specular = scene.light.color * best.material.albedo[1] * spec_i * light_intensity;

            (diffuse, specular, shadow)
        };

        // sombra
        let shadow = self.cast_shadow(&best, &scene.light, &scene.objects);
        let light_intensity = scene.light.intensity * (1.0 - shadow);

        let diff_i = n.dot(&light_dir).max(0.0).min(1.0);
        let diffuse = base_color * best.material.albedo[0] * diff_i * light_intensity;

        let spec_i = view_dir.dot(&reflect_dir).max(0.0).powf(best.material.specular);
        let specular = scene.light.color * best.material.albedo[1] * spec_i * light_intensity;

        let mut local_col = diffuse + specular;

        let kr = best.material.kr;
        let kt = best.material.kt;
        let ior = best.material.ior.max(1.0);

        if kr > 0.0 || kt > 0.0 {
            let mut n_use = n;
            let into = ray_d.dot(&n_use) < 0.0;
            let (n1, n2) = if into { (1.0_f32, ior) } else { (ior, 1.0_f32) };
            if !into { n_use = -n_use; }
            
            //Fresnel
            let cosi = (-*ray_d).dot(&n_use).clamp(0.0, 1.0);
            let f0 = ((n2 - n1) / (n2 + n1)).powi(2);
            let kf = self.fresnel_schlick(cosi, f0);

            // Reflejo
            let r_dir = self.reflect(ray_d, &n_use).normalize();
            let r_org = best.point + n_use * SHADOW_BIAS;
            let r_col = self.shade(&r_org, &r_dir, scene, depth + 1);

            // Refraccion
            let t_col = if kt > 0.0 {
                let eta = n1 / n2;
                if let Some(t_dir) = self.refract(ray_d, &n_use, eta) {
                    let t_dir_n = t_dir.normalize();
                    let t_org = best.point - n_use * SHADOW_BIAS;
                    self.shade(&t_org, &t_dir_n, scene, depth + 1)
                } else {
                    r_col
                }
            } else {
                Color::new(0,0,0)
            };

            let refl_w = kr.max(kf);
            let tran_w = (kt * (1.0 - kf)).max(0.0);
            let base_w = (1.0 - refl_w - tran_w).max(0.0);

            local_col = local_col * base_w + r_col * refl_w + t_col * tran_w;
        }

        local_col

    }

    // paralelo
    pub fn render_parallel(&self, fb: &mut Framebuffer, scene: &Scene, camera: &Camera) {
        let snap = camera.snapshot_for(fb.width, fb.height);

        let w = fb.width;
        let h = fb.height;

        // Procesa cada fila en paralelo
        fb.buffer
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let sy = -(2.0 * y as f32) / h as f32 + 1.0;
            let screen_y = sy * snap.persp;

            for x in 0..w {
                let sx = (2.0 * x as f32) / w as f32 - 1.0;
                let screen_x = sx * snap.aspect * snap.persp;

                let dir = (snap.x * screen_x + snap.y * screen_y + snap.z).normalize();
                let col = self.shade(&snap.eye, &dir, scene, 0).to_hex();

                row[x] = col;
            }
        });
    }

    #[allow(dead_code)]
    // secuencial
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

                let color = self.shade(&origin, &dir_world, scene, 0);
                row[x] = color.to_hex();
            }
            fb.write_row(y, &row);
        }
    }
}
