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
mod ring;
mod orbit;
mod controller;

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
use ring::Ring;
use orbit::Orbit;
use controller::Controller;

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
    let uranus_rings_texture = Arc::new(Texture::from_file("assets/uranus_rings.jpeg"));
    let saturn_rings_texture = Arc::new(Texture::from_file("assets/saturn_rings.jpg"));

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

    let mercury_material = Arc::new(
        Material::new(
            Color::new(201, 187, 172), // Naranja palido: rgba(201, 187, 172)
            30.0,                       // Specular moderado
            [0.7, 0.3],                // Albedo
            0.1,                        // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
    );

    let venus_material = Arc::new(
        Material::new(
            Color::new(243, 143, 11), //Naranja: rgba(243, 143, 11)
            25.0,                       // Specular bajo
            [0.6, 0.4],                // Albedo
            0.1,                        // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
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

    let mars_material = Arc::new(
        Material::new(
            Color::new(188, 39, 50), // Rojo oxidado: rgba(188, 39, 50)
            20.0,                      // Specular bajo
            [0.7, 0.3],               // Albedo
            0.1,                       // Poca reflectividad
            0.0,                       // Sin transparencia
            1.0,                       // IOR
        )
    );

    let jupiter_material = Arc::new(
        Material::new(
            Color::new(210, 180, 140), // Marrón claro: rgba(210, 180, 140)
            30.0,                       // Specular moderado
            [0.8, 0.2],                // Albedo
            0.1,                        // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
    );

    // Material de Saturno (sin texturas por ahora)
    let saturn_material = Arc::new(
        Material::new(
            Color::new(234, 214, 184), // Color beige/amarillento
            25.0,                       // Specular
            [0.8, 0.2],                // Albedo
            0.05,                       // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
    );

    let uranus_material = Arc::new(
        Material::new(
            Color::new(100, 234, 253), // Azul claro: rgb(100, 234, 253)
            35.0,                       // Specular
            [0.8, 0.2],                // Albedo
            0.1,                        // Poca reflectividad
            0.0,                        // Sin transparencia
            1.0,                        // IOR
        )
    );

    // material para anillos (puedes darle textura)
    let saturn_ring_material = Arc::new(
        Material::new(
            Color::new(200,180,150), 
            35.0, 
            [0.9,0.1], 
            0.0, 
            0.0, 
            1.0
        ).with_albedo_map(saturn_rings_texture.clone())
        .with_albedo_tiling(1.0, 1.0)
        .no_shadow()
    );

    let uranus_ring_material = Arc::new(
        Material::new(
            Color::new(180,200,220), 
            40.0, 
            [0.9,0.1], 
            0.2, 
            0.2, 
            1.0
        ).with_albedo_map(uranus_rings_texture.clone())
        .with_albedo_tiling(1.0, 1.0)
        .no_shadow()
    );

    // material para lunas (simple gris)
    let moon_material = Arc::new(
        Material::new(
            Color::new(200,200,200), 
            20.0, 
            [0.7,0.3], 
            0.0, 
            0.0, 
            1.0
        )
    );

    // Configurar cámara estática
    let mut camera = Camera::new(
        vec3(0.0, 15.0, 38.0),  // Posición elevada y alejada
        vec3(0.0, 0.0, 0.0),   // Mirando al sol (centro)
        vec3(0.0, 1.0, 0.0),   // Up vector
    );

    let mut controller = controller::Controller::new(0.2, 0.01, 0.5);

    // Parámetros de órbita de los planetas
    let mercury_orbit_radius = 6.0;
    let mercury_orbit_speed = 0.55;

    let venus_orbit_radius = 8.0;
    let venus_orbit_speed = 0.45;

    let earth_orbit_radius = 10.0;
    let earth_orbit_speed = 0.3;

    let mars_orbit_radius = 12.0;
    let mars_orbit_speed = 0.25;

    let jupiter_orbit_radius = 15.0;
    let jupiter_orbit_speed = 0.12;

    let moon_orbit_radius = 1.8;
    let moon_orbit_speed = 0.08;

    let saturn_orbit_radius = 20.0;
    let saturn_orbit_speed = 0.08;

    let uranus_orbit_radius = 25.0;
    let uranus_orbit_speed = 0.05;


    // Configurar iluminación desde el sol
    let lights = vec![
        Light::new(
            vec3(0.0, 0.0, 0.0), // Luz en el centro (sol)
            Color::new(255, 255, 230),
            5.5,
        ),
        Light::new(
            vec3(0.0, 19.0, 25.0),
            Color::new(255, 255, 230),
            1.6,
        )
    ];

    // Crear skybox de estrellas
    let stars = Stars::new(4000, WIDTH, HEIGHT, &camera);

    // Parametros de anillos
    let saturn_radius = 1.6;
    let uranus_radius = 1.0;

    let saturn_ring_inner = saturn_radius * 1.2;
    let saturn_ring_outer = saturn_radius * 2.0;

    let uranus_ring_inner = uranus_radius * 0.8;
    let uranus_ring_outer = uranus_radius * 1.5;

    let renderer = RenderPipeline::new();

    let start_time = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        controller.update(&window, &mut camera);

        // Calcular tiempo transcurrido
        let time = start_time.elapsed().as_secs_f32();

        // Calcular posiciones orbitales
        let mercury_angle = time * mercury_orbit_speed;
        let mercury_pos = vec3(
            mercury_orbit_radius * mercury_angle.cos(),
            0.0,
            mercury_orbit_radius * mercury_angle.sin(),
        );

        let venus_angle = time * venus_orbit_speed;
        let venus_pos = vec3(
            venus_orbit_radius * venus_angle.cos(),
            0.0,
            venus_orbit_radius * venus_angle.sin(),
        );

        let earth_angle = time * earth_orbit_speed;
        let earth_pos = vec3(
            earth_orbit_radius * earth_angle.cos(),
            0.0,
            earth_orbit_radius * earth_angle.sin(),
        );

        let mars_angle = time * mars_orbit_speed;
        let mars_pos = vec3(
            mars_orbit_radius * mars_angle.cos(),
            0.0,
            mars_orbit_radius * mars_angle.sin(),
        );

        let jupiter_angle = time * jupiter_orbit_speed;
        let jupiter_pos = vec3(
            jupiter_orbit_radius * jupiter_angle.cos(),
            0.0,
            jupiter_orbit_radius * jupiter_angle.sin(),
        );

        let saturn_angle = time * saturn_orbit_speed;
        let saturn_pos = vec3(
            saturn_orbit_radius * saturn_angle.cos(),
            0.0,
            saturn_orbit_radius * saturn_angle.sin(),
        );

        let uranus_angle = time * uranus_orbit_speed;
        let uranus_pos = vec3(
            uranus_orbit_radius * uranus_angle.cos(),
            0.0,
            uranus_orbit_radius * uranus_angle.sin(),
        );

        // orbitas de los anillos y lunas
        let saturn_orbit = Orbit::new(
            vec3(0.0, 0.0, 0.0),
            saturn_orbit_radius,
            saturn_orbit_speed
        ).with_tilt(26.7);

        let uranus_orbit = Orbit::new(
            vec3(0.0, 0.0, 0.0),
            uranus_orbit_radius,
            uranus_orbit_speed
        ).with_tilt(97.8);

        let moon_orbit = Orbit::new(
            earth_pos, 
            moon_orbit_radius, 
            moon_orbit_speed
        ).with_phase(45.0);

        let moon_orbit_radius = 1.2;
        let moon_orbit_speed = 0.08;

        let phobos_orbit_radius = 0.9;
        let phobos_orbit_speed = 1.2;

        let deimos_orbit_radius = 1.4;
        let deimos_orbit_speed = 0.6;

        let phobos_orbit = Orbit::new(
            mars_pos, 
            phobos_orbit_radius,
            phobos_orbit_speed
        );

        let deimos_orbit = Orbit::new(
            mars_pos,
            deimos_orbit_radius,
            deimos_orbit_speed
        );

        // Crear el Sol en el centro (estático)
        let sun = Arc::new(Sphere::new(
            vec3(0.0, 0.0, 0.0),
            1.9,  // Sol más pequeño para mejor proporción
            sun_material.clone(),
        ));

        // Crear la Lunas
        let mut moon_orbit_local = moon_orbit.clone();
        moon_orbit_local.center = earth_pos;
        let moon_pos = moon_orbit_local.position_at(time);

        let mut phobos_orbit_local = phobos_orbit.clone();
        phobos_orbit_local.center = mars_pos;

        let mut deimos_orbit_local = deimos_orbit.clone();
        deimos_orbit_local.center = mars_pos;

        let phobos_pos = phobos_orbit_local.position_at(time);
        let deimos_pos = deimos_orbit_local.position_at(time);

        // Anillos
        let saturn_ring_normal = {
            let tilt = saturn_orbit.tilt;
            vec3(0.0, tilt.cos(), tilt.sin()).normalize()
        };
        let uranus_ring_normal = {
            let tilt = uranus_orbit.tilt;
            vec3(0.0, tilt.cos(), tilt.sin()).normalize()
        };

        // Crear planetas en posiciones orbitales (posicion, radio, material)
        let mercury = Arc::new(Sphere::new(mercury_pos, 0.2, mercury_material.clone()));
        let venus   = Arc::new(Sphere::new(venus_pos,   0.6, venus_material.clone()));
        let earth   = Arc::new(Sphere::new(earth_pos,   0.7, earth_material.clone()));
        let mars    = Arc::new(Sphere::new(mars_pos,    0.3, mars_material.clone()));
        let jupiter = Arc::new(Sphere::new(jupiter_pos, 1.4, jupiter_material.clone()));
        let saturn  = Arc::new(Sphere::new(saturn_pos,  saturn_radius, saturn_material.clone()));
        let uranus  = Arc::new(Sphere::new(uranus_pos,  uranus_radius, uranus_material.clone()));

        // Crear lunas y anillos
        let moon = Arc::new(Sphere::new(moon_pos, 0.27, moon_material.clone()));
        let phobos = Arc::new(Sphere::new(phobos_pos, 0.12, moon_material.clone()));
        let deimos = Arc::new(Sphere::new(deimos_pos, 0.09, moon_material.clone()));
        let saturn_ring = Arc::new(Ring::new(saturn_pos,
            saturn_ring_normal,
            saturn_ring_inner,
            saturn_ring_outer,
            saturn_ring_material.clone(),
        ));

        let uranus_ring = Arc::new(Ring::new(
            uranus_pos,
            uranus_ring_normal,
            uranus_ring_inner,
            uranus_ring_outer,
            uranus_ring_material.clone(),
        ));

        // Crear bounding boxes
        let sun_bbox = AABB::from_sphere(vec3(0.0, 0.0, 0.0), 1.9);
        let mercury_bbox = AABB::from_sphere(mercury_pos, 0.2);
        let venus_bbox   = AABB::from_sphere(venus_pos, 0.6);
        let earth_bbox   = AABB::from_sphere(earth_pos, 0.8);
        let mars_bbox    = AABB::from_sphere(mars_pos, 0.4);
        let jupiter_bbox = AABB::from_sphere(jupiter_pos, 1.4);
        let saturn_bbox  = AABB::from_sphere(saturn_pos,  1.6);
        let uranus_bbox  = AABB::from_sphere(uranus_pos, 1.0);
        let moon_bbox    = AABB::from_sphere(moon_pos, 0.27);
        let phobos_bbox = AABB::from_sphere(phobos_pos, 0.12);
        let deimos_bbox = AABB::from_sphere(deimos_pos, 0.09);
        let saturn_ring_bbox = AABB::from_sphere(saturn_pos, saturn_ring_outer);
        let uranus_ring_bbox = AABB::from_sphere(uranus_pos, uranus_ring_outer);

        // Crear escena con órbitas
        let objects: Vec<Arc<dyn intersect::RayIntersect>> = vec![
            sun as Arc<dyn intersect::RayIntersect>,
            mercury as Arc<dyn intersect::RayIntersect>,
            venus as Arc<dyn intersect::RayIntersect>,
            earth as Arc<dyn intersect::RayIntersect>,
            mars as Arc<dyn intersect::RayIntersect>,
            jupiter as Arc<dyn intersect::RayIntersect>,
            saturn as Arc<dyn intersect::RayIntersect>,
            uranus as Arc<dyn intersect::RayIntersect>,
            moon as Arc<dyn intersect::RayIntersect>,
            phobos as Arc<dyn intersect::RayIntersect>,
            deimos as Arc<dyn intersect::RayIntersect>,
            saturn_ring as Arc<dyn intersect::RayIntersect>,
            uranus_ring as Arc<dyn intersect::RayIntersect>,
        ];

        let bboxes = vec![sun_bbox, mercury_bbox, venus_bbox, earth_bbox, mars_bbox, jupiter_bbox, saturn_bbox, uranus_bbox, moon_bbox, phobos_bbox, deimos_bbox, saturn_ring_bbox, uranus_ring_bbox];
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