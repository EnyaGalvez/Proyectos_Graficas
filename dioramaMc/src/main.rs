// src/main
use std::sync::Arc;
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
mod aabb;

use framebuffer::Framebuffer;
use intersect::RayIntersect;
use shapes3d::{Cube, Wall, Stair, Facing, Orientation};
use color::Color;
use camera::Camera;
use light::Light;
use material::Material;
use texture::{Texture, TextureOptions};
use render::{Scene, RenderPipeline};
use aabb::AABB;

#[derive(Clone)]
pub struct SkinTexture {
    pub albedo: Texture,
    pub normal: Texture,
}

pub fn load_textures() -> HashMap<&'static str, SkinTexture> {
    let mut texture_table = HashMap::new();

    let albedo_opts = TextureOptions {
        generate_mips: true,
        max_w: 512,
        max_h: 512,
        ..Default::default()
    };

    let normal_opts = TextureOptions {
        generate_mips: true,
        max_w: 256,
        max_h: 256,
        ..Default::default()
    };

    texture_table.insert("wood", SkinTexture {
        albedo: Texture::from_file_with("assets/wood-texture.jpg", albedo_opts),
        normal: Texture::from_file_with("assets/wood-normal.jpg", normal_opts),
    });

    texture_table.insert("brick", SkinTexture {
        albedo: Texture::from_file_with("assets/brick-texture.jpg", albedo_opts),
        normal: Texture::from_file_with("assets/brick-normal.jpg", normal_opts),
    });

    texture_table.insert("stone", SkinTexture{
        albedo: Texture::from_file_with("assets/stone-texture.jpg", albedo_opts),
        normal: Texture::from_file_with("assets/stone-normal.jpg", normal_opts),
    });

    texture_table.insert("wool", SkinTexture{
        albedo: Texture::from_file_with("assets/rug-texture.jpg", albedo_opts),
        normal: Texture::from_file_with("assets/rug-normal.jpg", normal_opts),
    });
    
    texture_table
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
    let texture_table = load_textures();

    // materials
    // Madera
    let wood_texture = texture_table.get("wood").unwrap();
    let wood = Arc::new(Material::new(
        Color::new(181, 140, 90), 
        16.0, 
        [0.85, 0.05]
    ).with_albedo_map( 
        Arc::new(wood_texture.albedo.clone()), 0.5, 0.5
    ).with_normal_map(
        Arc::new(wood_texture.normal.clone()),
        0.5, 0.5,
    ).with_reflectance(0.02).with_transparency(0.0, 1.0));


    // Ladrillo
    let brick_texture = texture_table.get("brick").unwrap();
    let brick = Arc::new(
        Material::new(
            Color::new(180,180,180), 
            12.0,
            [0.9, 0.05]
        ).with_albedo_map(
            Arc::new(brick_texture.albedo.clone()),
            0.4, 0.4
        ).with_normal_map(
            Arc::new(brick_texture.normal.clone()),
            0.4, 0.4,
        ).with_reflectance(0.01).with_transparency(0.0, 1.0)
    );

    // Piedra
    let stone_texture = texture_table.get("stone").unwrap();
    let stone = Arc::new(
        Material::new(
            Color::new(190,190,190), 
            28.0,
            [0.8, 0.10]
        ).with_albedo_map(
            Arc::new(stone_texture.albedo.clone()),
            0.5, 0.5
        ).with_normal_map(
            Arc::new(stone_texture.normal.clone()),
            0.5, 0.5,
        ).with_reflectance(0.03).with_transparency(0.0, 1.0)
    );

    // Lana
    let wool_texture = texture_table.get("wool").unwrap();
    let wool = Arc::new(
            Material::new(
            Color::new(255, 153, 204), 
            10.0,
            [0.9, 0.1]
        ).with_albedo_map(
            Arc::new(wool_texture.albedo.clone()),
            0.5, 0.5
        ).with_normal_map(
            Arc::new(wool_texture.normal.clone()),
            0.5, 0.5,
        ).with_reflectance(0.0).with_transparency(0.0, 1.0)
    );

    let glass = Arc::new(
        Material::new(
            Color::new(2,2,3), 
            96.0,
            [0.02, 0.10]
        ).with_reflectance(0.04).with_transparency(0.9,1.5)
    );

    // Comprobacion de carga de texturas
    /*
    println!("brick albedo: {}x{}", brick_texture.albedo.w, brick_texture.albedo.h);
    println!("wood albedo: {}x{}", wood_texture.albedo.w, wood_texture.albedo.h);
    println!("\nbrick normal: {}x{}", brick_texture.normal.w, brick_texture.normal.h);
    println!("wood normal: {}x{}", wood_texture.normal.w, wood_texture.normal.h);*/

    const SCALE: f32 = 2.0 / 3.0;
    let edge = 1.6 * SCALE;

    // suelo
    let floor_base = wood.clone();
    let floor_sx = 10.0;
    let floor_sz = 10.0;
    let floor_thickness = 0.10;

    let obj_half_h = (1.6 * SCALE) * 0.5;

    let floor_center_y = -obj_half_h - floor_thickness * 0.5;

    let floor = Wall::from_center_dims(
        Vec3::new(0.0, floor_center_y, 0.0), 
        floor_sx, floor_thickness, floor_sz, 
        floor_base.clone()
    ).with_tiling(6.0, 6.0);

    //techo
    let roof = Wall::from_center_dims(
        Vec3::new(0.0, floor_center_y * 5.0, 0.0),
        floor_sx, floor_thickness, floor_sz, 
        floor_base.clone()
    ).with_tiling(6.0, 6.0);

    // fireplace
    // cubos de piedra
    let stone_rtcube = Cube::from_center_size(
        Vec3::new(1.1, 1.0, 4.2), 
        edge, 
        stone.clone()
    );

    let stone_rbcube = Cube::from_center_size(
        Vec3::new(1.1, 0.0, 4.23), 
        edge, 
        stone.clone()
    );

    let stone_ltcube = Cube::from_center_size(
        Vec3::new(3.1, 1.0, 4.23), 
        edge, 
        stone.clone()
    );

    let stone_lbcube = Cube::from_center_size(
        Vec3::new(3.1, 0.0, 4.23), 
        edge, 
        stone.clone()
    );

    let stone_tbcube = Cube::from_center_size(
        Vec3::new(2.1, 2.0, 4.23), 
        edge, 
        stone.clone()
    );

    let stone_tccube = Cube::from_center_size(
        Vec3::new(2.1, 3.0, 4.23), 
        edge, 
        stone.clone()
    );

    let stone_ttcube = Cube::from_center_size(
        Vec3::new(2.1, 4.0, 4.23), 
        edge, 
        stone.clone()
    );

    // cubos de atras (fireplace)
    let stone_bbcube = Cube::from_center_size(
        Vec3::new(2.1, 0.0, 5.23), 
        edge, 
        stone.clone()
    );

    let stone_btcube = Cube::from_center_size(
        Vec3::new(2.1, 1.0, 5.23), 
        edge, 
        stone.clone()
    );

    // escaleras de decoracion
    let stone_rstair = Stair::from_center_edge_control(
        Vec3::new(1.1, 2.0, 4.23),
        edge,
        stone.clone(),
        Facing::Right,
        Orientation::Upright
    ).with_tiling(0.5, 0.5);

    let stone_lstair = Stair::from_center_edge_control(
        Vec3::new(3.1, 2.0, 4.23),
        edge,
        stone.clone(),
        Facing::Left,
        Orientation::Upright
    ).with_tiling(0.5, 0.5);

    let stone_board1 = Stair::from_center_edge_control(
        Vec3::new(1.1, 1.0, 3.23),
        edge,
        stone.clone(),
        Facing::Forward,
        Orientation::UpsideDown
    ).with_tiling(0.5, 0.5);

    let stone_board2 = Stair::from_center_edge_control(
        Vec3::new(2.1, 1.0, 3.23),
        edge,
        stone.clone(),
        Facing::Forward,
        Orientation::UpsideDown
    ).with_tiling(0.5, 0.5);

    let stone_board3 = Stair::from_center_edge_control(
        Vec3::new(3.1, 1.0, 3.23),
        edge,
        stone.clone(),
        Facing::Forward,
        Orientation::UpsideDown
    ).with_tiling(0.5, 0.5);

    //paredes
    let f_rwall = Wall::from_center_dims(
        Vec3::new(0.6, 0.0, 4.9), 
        edge * 2.22, // ancho
        edge * 8.88, // alto
        0.6 * SCALE, // grosor
        brick.clone()
    ).with_tiling(1.0, 4.0);

    let f_lwall = Wall::from_center_dims(
        Vec3::new(3.6, 0.0, 4.9), 
        edge * 2.22, // ancho
        edge * 8.88, // alto
        0.6 * SCALE, // grosor
        brick.clone()
    ).with_tiling(1.0, 4.0);

    let left_wall = Wall::from_center_dims(
        Vec3::new(
            (floor_sx * 0.5) - (0.6 * SCALE * 0.5),
            floor_center_y + ((edge * 6.88) * 0.5),
            0.0
        ), 
        0.6 * SCALE, // ancho
        edge * 6.88, // alto
        floor_sz, // grosor
        brick.clone()
    ).with_tiling(3.0, 4.0);

    let back_wall = Wall::from_center_dims(
        Vec3::new(
            0.0,
            floor_center_y + ((edge * 6.88) * 0.5),
            -((floor_sz * 0.5) + (0.6 * SCALE * 0.5))
        ), 
        floor_sx, // ancho
        edge * 6.88, // alto
        0.6 * SCALE, // grosor
        brick.clone()
    ).with_tiling(3.0, 4.0);

    let wool_cube = Cube::from_center_size(
        Vec3::new(-1.0, 0.0, 0.0), 
        1.6 * SCALE, 
        wool
    );
    
    // ventana
    let glass_wall = Wall::from_center_dims(
        Vec3::new(-2.6, 0.0, 4.9), 
        edge * 2.22, 
        edge * 8.88, 
        0.6 * SCALE, 
        glass
    );

    let stair = Stair::from_center_edge(
        Vec3::new(3.0, 0.0, 0.0), 
        1.6 * SCALE, 
        wood, 
        false
    ).with_tiling(1.0, 1.0);

    // luz y escena
    let light = Light::new(
        Vec3::new(-3.0, 6.0, -3.0),
        Color::new(255, 255, 204),
        1.0
    );

    /// aabb
    let mut objects: Vec<Arc<dyn RayIntersect>> = Vec::new();
    let mut bboxes: Vec<AABB> = Vec::new();

    fn push_wall(objects: &mut Vec<Arc<dyn RayIntersect>>, bboxes: &mut Vec<AABB>, w: Wall) {
        let bbox = AABB { min: w.min, max: w.max };
        objects.push(Arc::new(w) as Arc<dyn RayIntersect>);
        bboxes.push(bbox);
    }

    fn push_cube(objects: &mut Vec<Arc<dyn RayIntersect>>, bboxes: &mut Vec<AABB>, c: Cube) {
        let bbox = AABB { min: c.min, max: c.max };
        objects.push(Arc::new(c) as Arc<dyn RayIntersect>);
        bboxes.push(bbox);
    }

    fn push_stair(objects: &mut Vec<Arc<dyn RayIntersect>>, bboxes: &mut Vec<AABB>, s: Stair) {
        let lower = AABB { min: s.lower.min, max: s.lower.max };
        let upper = AABB { min: s.upper.min, max: s.upper.max };
        let bbox = AABB::union(lower, upper);
        objects.push(Arc::new(s) as Arc<dyn RayIntersect>);
        bboxes.push(bbox);
    }

    // Construccion de escena 
    // Suelo y techo
    push_wall(&mut objects, &mut bboxes, floor);
    push_wall(&mut objects, &mut bboxes, roof);

    // Fireplace - cubos de piedra
    push_cube(&mut objects, &mut bboxes, stone_rtcube);
    push_cube(&mut objects, &mut bboxes, stone_rbcube);
    push_cube(&mut objects, &mut bboxes, stone_ltcube);
    push_cube(&mut objects, &mut bboxes, stone_lbcube);
    push_cube(&mut objects, &mut bboxes, stone_tbcube);
    push_cube(&mut objects, &mut bboxes, stone_tccube);
    push_cube(&mut objects, &mut bboxes, stone_ttcube);

    // Cubos de atrás
    push_cube(&mut objects, &mut bboxes, stone_bbcube);
    push_cube(&mut objects, &mut bboxes, stone_btcube);

    // Escaleras de decoración
    push_stair(&mut objects, &mut bboxes, stone_rstair);
    push_stair(&mut objects, &mut bboxes, stone_lstair);
    push_stair(&mut objects, &mut bboxes, stone_board1);
    push_stair(&mut objects, &mut bboxes, stone_board2);
    push_stair(&mut objects, &mut bboxes, stone_board3);

    // Paredes
    push_wall(&mut objects, &mut bboxes, f_rwall);
    push_wall(&mut objects, &mut bboxes, f_lwall);
    push_wall(&mut objects, &mut bboxes, left_wall);
    push_wall(&mut objects, &mut bboxes, back_wall);

    // Otros
    push_cube(&mut objects, &mut bboxes, wool_cube);
    push_wall(&mut objects, &mut bboxes, glass_wall);

    // Escalera principal
    push_stair(&mut objects, &mut bboxes, stair);

    let scene = Scene::new(objects, bboxes, light);

    let renderer = RenderPipeline::new();

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(4.0, 1.5, 0.0), // eye
        Vec3::new(0.0, 1.0, 5.0), // look at
        Vec3::new(0.0, 1.0, 0.0), // up
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
        renderer.render_parallel(&mut framebuffer, &scene, &camera);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}