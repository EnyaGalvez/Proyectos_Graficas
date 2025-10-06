// src/main

use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod intersect;
mod cube; 
mod color;
mod camera;
mod light;
mod material;
mod texture;

use framebuffer::Framebuffer;
use cube::Cube;
use color::Color;
use intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;

const SHADOW_BIAS: f32 = 1e-4;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let offset_normal = intersect.normal * SHADOW_BIAS;
    let shadow_ray_origin = if light_dir.dot(&intersect.normal) < 0.0 {
        intersect.point - offset_normal
    } else {
        intersect.point + offset_normal
    };

    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(0, 0, 26);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    diffuse + specular
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    // random number generator
    // let mut rng = rand::thread_rng();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective 
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 1000;
    let window_height = 800;
    let framebuffer_width = 1000;
    let framebuffer_height = 800;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Diorama - Proyecto 2",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    let pink = Material::new(
        Color::new(255, 153, 204),
        10.0,
        [0.9, 0.1],
    );

    let objects = [
        Cube::from_center_size(Vec3::new(0.0, 0.0, 0.0), 1.6, pink),
    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(5.0, 3.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI/50.0;

    let light = Light::new(
        Vec3::new(4.0, 6.0, 3.0),
        Color::new(255, 255, 204),
        1.0
    );

    // posicion y movimiento de camara
    let yaw_speed = PI / 1.5; // rad/s  (flechas izq/der)
    let pitch_speed = PI / 3.0; // rad/s  (flechas arr/abajo)
    let pan_speed = 2.5; // unidades/seg (WASD y RF)
    let dolly_speed = 3.5; // unidades/seg (Q/E)
    let fast_multiplier = 3.0; // Shift
    let slow_multiplier = 0.35; // Ctrl


    let mut last_time = Instant::now();

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        
        // factor de velocidad
        let mut speed_factor = 1.0;
        if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
            speed_factor *= fast_multiplier;
        }
        if window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl) {
            speed_factor *= slow_multiplier;
        }

        // Orbita con flechas
        let dyaw = yaw_speed * dt * speed_factor;
        let dpitch = pitch_speed * dt * speed_factor;

        if window.is_key_down(Key::Left) { camera.orbit(dyaw, 0.0); }
        if window.is_key_down(Key::Right) { camera.orbit(-dyaw, 0.0); }
        if window.is_key_down(Key::Up) { camera.orbit(0.0, dpitch); }
        if window.is_key_down(Key::Down) { camera.orbit(0.0, -dpitch); }

        // Pan en X/Y de cámara (WASD)
        let p = pan_speed * dt * speed_factor;
        if window.is_key_down(Key::A) { camera.pan(-p, 0.0); }
        if window.is_key_down(Key::D) { camera.pan( p, 0.0); }
        if window.is_key_down(Key::W) { camera.pan(0.0,  p); }
        if window.is_key_down(Key::S) { camera.pan(0.0, -p); }

        // Dolly en Z de cámara (Q/E)
        let dz = dolly_speed * dt * speed_factor;
        if window.is_key_down(Key::Q) { camera.dolly(-dz); }
        if window.is_key_down(Key::E) { camera.dolly( dz); }
    }
}