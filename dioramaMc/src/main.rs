// src/main

use nalgebra_glm::Vec3;
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use std::collections::HashMap;

mod framebuffer;
mod intersect;
mod shapes3d;
mod color;
mod camera;
mod light;
mod material;
mod texture;
mod render;

use framebuffer::Framebuffer;
use shapes3d::{Cube, Wall, Stair};
use color::Color;
use camera::Camera;
use light::Light;
use material::Material;
use texture::Texture;
use render::{Scene, RenderPipeline};

#[derive(Clone)]
pub struct SkinTexture {
    pub albedo: Texture,
    pub normal: Texture,
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    framebuffer.set_background_color(Color::from_hex(0x001018).to_hex());

    let mut window = Window::new(
        "Diorama - Proyecto 2",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    // textures from images
    let mut texture_table: HashMap<&str, SkinTexture> = HashMap::new();

    texture_table.insert("wood", SkinTexture {
        albedo: Texture::from_file("assets/wood-texture.jpg"),
        normal: Texture::from_file("assets/wood-normal.jpg"),
    });

    texture_table.insert("brick", SkinTexture {
        albedo: Texture::from_file("assets/brick-texture.jpg"),
        normal: Texture::from_file("assets/brick-normal.jpg"),
    });

    // materials
    let pink = Material::new(
        Color::new(255, 153, 204),
        10.0,
        [0.9, 0.1],
    );

    let wood_texture = texture_table.get("wood").unwrap();
    let wood = Material::new(
        Color::new(181,140, 90),
        8.0,
        [0.9, 0.1]
    ).with_albedo_map(
        wood_texture.albedo.clone(),
        0.5, 0.5
    );


    let brick_texture = texture_table.get("brick").unwrap();
    let brick = Material::new(
        Color::new(180,180,180), 
        32.0,
        [0.8, 0.2]
    ).with_albedo_map(
        brick_texture.albedo.clone(),
        0.5, 0.5
    );

    // Comprobacion de carga de texturas
    println!("brick albedo: {}x{}", brick_texture.albedo.w, brick_texture.albedo.h);
    println!("wood albedo: {}x{}", wood_texture.albedo.w, wood_texture.albedo.h);
    println!("\nbrick normal: {}x{}", brick_texture.normal.w, brick_texture.normal.h);
    println!("wood normal: {}x{}", wood_texture.normal.w, wood_texture.normal.h);

    // Objects (cubo, pared, escalera)
    const SCALE: f32 = 2.0 / 3.0;
    let cube = Cube::from_center_size(Vec3::new(-2.0, 0.0, 0.0), 1.6 * SCALE, pink);

    let v_slap = Wall::from_center_dims(Vec3::new(0.0, 0.0, 0.0), 1.6 * SCALE, 1.6 * SCALE, 0.6 * SCALE, brick);

    let stair = Stair::from_center_edge(Vec3::new(2.0, 0.0, 0.0), 1.6 * SCALE, wood, false);

    let light = Light::new(
        Vec3::new(5.0, 6.0, 4.0),
        Color::new(255, 255, 204),
        1.0
    );

    let scene = Scene::new(
            vec![
            Box::new(cube) as Box<dyn crate::intersect::RayIntersect>,
            Box::new(v_slap) as Box<dyn crate::intersect::RayIntersect>,
            Box::new(stair) as Box<dyn crate::intersect::RayIntersect>,
        ],
        light,
    );

    let renderer = RenderPipeline::new();

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(5.0, 3.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // posicion y movimiento de camara
    let yaw_speed = PI / 1.5;
    let pitch_speed = PI / 3.0;
    let pan_speed = 2.5;
    let dolly_speed = 3.5;
    let fast_multiplier = 3.0;
    let slow_multiplier = 0.35;

    let mut last_time = Instant::now();

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

        framebuffer.clear();
        renderer.render(&mut framebuffer, &scene, &camera);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}