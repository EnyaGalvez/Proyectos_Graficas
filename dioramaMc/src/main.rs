// src/main

use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod intersect;
mod shapes3d;
mod color;
mod camera;
mod light;
mod material;
mod texture;

use framebuffer::Framebuffer;
use shapes3d::{Cube, Wall, Stair};
use color::Color;
use intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;
use texture::Texture;

const SHADOW_BIAS: f32 = 1e-4;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
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
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for obj in objects {
        let i = obj.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(0, 0, 26);
    }

    // albedo
    let base_color = if let Some(tex) = &intersect.material.albedo_map {
        if intersect.has_uv {
            let (u, v) = intersect.uv;
            tex.sample_tiled(u, v, intersect.material.tiling)
        } else {
            intersect.material.diffuse
        }
    } else {
        intersect.material.diffuse
    };

    // normal map
    let mut n = intersect.normal;
    if let (true, Some(nmap)) = (intersect.has_tangent, &intersect.material.normal_map) {
        if intersect.has_uv {
            let (u, v) = intersect.uv;
            let nt = nmap.sample_normal_tangent(u, v, intersect.material.tiling); // (x,y,z) [-1,1]
            let t = nalgebra_glm::normalize(&intersect.tangent);
            let b = nalgebra_glm::normalize(&intersect.bitangent);
            let n0 = nalgebra_glm::normalize(&n);
            n = nalgebra_glm::normalize(&(t * nt.x + b * nt.y + n0 * nt.z));
        }
    }

    // luz
    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &n);

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = n.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = base_color * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    return diffuse + specular
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light) {
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
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

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

    // textures
    let brick_albedo = Texture::from_file("assets/brick-texture.jpg");
    let brick_normal = Texture::from_file("assets/brick-normal.jpg");

    let wood_albedo = Texture::from_file("assets/wood-texture.jpg");
    let wood_normal = Texture::from_file("assets/wood-normal.jpg");

    println!("brick: {}x{}", brick_albedo.w, brick_albedo.h);
    println!("wood:  {}x{}", wood_albedo.w,  wood_albedo.h);

    // materials
    let pink = Material::new(
        Color::new(255, 153, 204),
        10.0,
        [0.9, 0.1],
    );

    let wood = Material::new(
        Color::new(181,140, 90),
        8.0,
        [0.9, 0.1]
    ).with_albedo_map(wood_albedo.clone(), 2.0).with_normal_map(wood_normal.clone());

    let stone = Material::new(
        Color::new(180,180,180), 
        32.0,
        [0.8, 0.2]
    ).with_albedo_map(brick_albedo.clone(), 2.0).with_normal_map(brick_normal.clone());

    // Objects (cubo, pared, escalera)
    let cube = Cube::from_center_size(Vec3::new(-2.0, 0.0, 0.0), 1.6, pink);

    let v_slap = Wall::from_center_dims(Vec3::new(0.0, 0.0, 0.0), 1.6, 1.6, 0.6, stone);

    let stair = Stair::from_center_edge(Vec3::new(2.0, 0.0, 0.0), 1.6, wood, false);

    let scene: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(cube),
        Box::new(v_slap),
        Box::new(stair),
    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(5.0, 3.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(
        Vec3::new(6.0, 8.0, 6.0),
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

    let mut dirty = true;


    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        if window.is_key_down(Key::Escape) {
            break;
        }

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

        let mut moved = false;

        if window.is_key_down(Key::Left) { 
            camera.orbit(dyaw, 0.0);
            moved = true;
        }
        if window.is_key_down(Key::Right) { 
            camera.orbit(-dyaw, 0.0);
            moved = true;
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, dpitch);
            moved = true;
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, -dpitch);
            moved = true;
        }

        // Pan en X/Y de cámara (WASD)
        let p = pan_speed * dt * speed_factor;
        if window.is_key_down(Key::A) {
            camera.pan(-p, 0.0);
            moved = true;
        }
        if window.is_key_down(Key::D) {
            camera.pan( p, 0.0);
            moved = true;
        }
        if window.is_key_down(Key::W) {
            camera.pan(0.0,  p);
            moved = true;
        }
        if window.is_key_down(Key::S) {
            camera.pan(0.0, -p);
            moved = true;
        }

        // Dolly en Z de cámara (Q/E)
        let dz = dolly_speed * dt * speed_factor;
        if window.is_key_down(Key::Q) {
            camera.dolly(-dz);
            moved = true;
        }
        if window.is_key_down(Key::E) {
            camera.dolly( dz);
            moved = true;
        }

        

        if moved { dirty = true; }

        if dirty {
            framebuffer.clear();
            render(&mut framebuffer, &scene, &camera, &light);
            dirty = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}