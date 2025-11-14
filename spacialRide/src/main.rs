// src/main.rs
mod camera;
mod color;
mod framebuffer;
mod intersect;
mod light;
mod material;
mod sphere;
mod texture;
mod render;
mod aabb;
mod stars;

use std::sync::Arc;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::vec3;

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use light::Light;
use material::Material;
use sphere::Sphere;
use texture::Texture;
use render::{Scene, RenderPipeline};
use aabb::AABB;
use stars::Stars;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let mut window = Window::new(
        "Sistema Solar",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Error al crear ventana: {}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));
    
    // Framebuffer
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(0x000000);

    // Cargar texturas
    let earth_texture = Arc::new(Texture::from_file("assets/earth.jpg"));

    // Material del Sol (emisivo, sin texturas)
    let sun_material = Arc::new(
        Material::new(
            Color::new(255, 220, 100), // Amarillo brillante
            100.0,                      // Muy especular
            [1.0, 0.5],                // Albedo alto
            0.0,                        // Sin reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        ).no_shadow()
    );

    // Material de la Tierra (con textura y normal map)
    let earth_material = Arc::new(
        Material::new(
            Color::new(70, 130, 180), // Color base azul
            50.0,                      // Specular
            [0.9, 0.1],               // Albedo [diffuse, specular]
            0.1,                       // Reflectividad
            0.0,                       // Transparencia
            1.0,                       // IOR
        ).with_albedo_map(earth_texture.clone())
        .with_albedo_tiling(1.0, 1.0)
    );

    // Material de Saturno (sin texturas por ahora)
    let saturn_material = Arc::new(
        Material::new(
            Color::new(234, 214, 184), // Color beige/amarillento
            40.0,                       // Specular
            [0.8, 0.2],                // Albedo
            0.05,                       // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
    );

    // Configurar cámara estática
    let camera = Camera::new(
        vec3(0.0, 15.0, 25.0),  // Posición elevada y alejada
        vec3(0.0, 0.0, 0.0),   // Mirando al sol (centro)
        vec3(0.0, 1.0, 0.0),   // Up vector
    );

    // Parámetros de órbita
    let earth_orbit_radius = 6.0;
    let earth_orbit_speed = 0.3; // Velocidad angular (radianes por segundo)
    
    let saturn_orbit_radius = 12.0;
    let saturn_orbit_speed = 0.15; // Más lento que la Tierra

    // Configurar iluminación desde el sol
    let lights = vec![
        Light::new(
            vec3(0.0, 0.0, 0.0), // Luz en el centro (sol)
            Color::new(255, 255, 230),
            5.5,
        ),
        Light::new(
            vec3(0.0, 15.0, 25.0),
            Color::new(255, 255, 230),
            2.0,
        )
    ];

    // Crear skybox de estrellas
    let stars = Stars::new(0.002);

    let renderer = RenderPipeline::new();

    let start_time = std::time::Instant::now();


    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Calcular tiempo transcurrido
        let time = start_time.elapsed().as_secs_f32();

        // Calcular posiciones orbitales
        let earth_angle = time * earth_orbit_speed;
        let earth_pos = vec3(
            earth_orbit_radius * earth_angle.cos(),
            0.0,
            earth_orbit_radius * earth_angle.sin(),
        );

        let saturn_angle = time * saturn_orbit_speed;
        let saturn_pos = vec3(
            saturn_orbit_radius * saturn_angle.cos(),
            0.0,
            saturn_orbit_radius * saturn_angle.sin(),
        );

        // Crear el Sol en el centro (estático)
        let sun = Arc::new(Sphere::new(
            vec3(0.0, 0.0, 0.0),
            1.5,  // Sol más pequeño para mejor proporción
            sun_material.clone(),
        ));

        // Crear planetas en posiciones orbitales
        let earth = Arc::new(Sphere::new(
            earth_pos,
            1.0,  // Tierra tamaño medio
            earth_material.clone(),
        ));

        let saturn = Arc::new(Sphere::new(
            saturn_pos,
            1.8,  // Saturno más grande que la Tierra
            saturn_material.clone(),
        ));

        // Crear bounding boxes con posiciones actualizadas
        let sun_bbox = AABB::from_sphere(vec3(0.0, 0.0, 0.0), 1.5);
        let earth_bbox = AABB::from_sphere(earth_pos, 1.0);
        let saturn_bbox = AABB::from_sphere(saturn_pos, 1.8);

        // Crear escena con órbitas
        let objects: Vec<Arc<dyn intersect::RayIntersect>> = vec![
            sun as Arc<dyn intersect::RayIntersect>,
            earth as Arc<dyn intersect::RayIntersect>,
            saturn as Arc<dyn intersect::RayIntersect>,
        ];

        let bboxes = vec![sun_bbox, earth_bbox, saturn_bbox];
        let scene = Scene::new(objects, bboxes, lights.clone(), stars.clone());

        // Limpiar y renderizar
        framebuffer.clear();
        renderer.render_parallel(&mut framebuffer, &scene, &camera);

        // Actualizar ventana
        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    println!("Programa finalizado");
}