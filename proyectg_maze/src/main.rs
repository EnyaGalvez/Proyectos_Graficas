// src/main.rs

use raylib::prelude::*;
use std::f32::consts::FRAC_PI_4;

mod maze;
mod render3d;
mod player;
mod caster;
mod controller;
mod textures;
mod sprites;
mod framebuffer;
mod minimap;
mod hud;

use maze::{load_maze, render_maze, find_first_free_cell};
use render3d::render3d;
use player::Player;
use caster::cast_ray;
use controller::process_input;
use textures::TextureManager;
use sprites::{Sprite, draw_sprites};
use framebuffer::{Framebuffer, calc_block_size_offset};
use minimap::draw_minimap;
use hud::draw_fps_top_left;

fn main() {
    let window_width: i32 = 1000;
    let window_height: i32 = 800;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Laberinto Verde")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    // Framebuffer
    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::BLACK);
    framebuffer.clear();

    let texman = TextureManager::new(&mut window, &raylib_thread);

    // Cargar mapa desde archivo txt
    let maze = load_maze("maze.txt");

    // Tamaño de bloque y offsets del mapa en pantalla
    let (block, offset_x, offset_y) =
        calc_block_size_offset(&maze, window_width as u32, window_height as u32);

    // Elegir spawn en celda libre
    let (spawn_x, spawn_y) = find_first_free_cell(&maze).unwrap_or((0, 0));
    
    // Crear jugador
    let mut player = Player {
        pos: Vector2::new(
            spawn_x as f32 + 0.5, spawn_y as f32 + 0.5
        ),
        a: FRAC_PI_4, // angulo de vista inicial (45)
    };

    let mut sprites_list = vec![
        Sprite {
            pos: Vector2::new(spawn_x as f32 + 3.5, spawn_y as f32 + 1.5), 
            tex_keys: vec!['c', 'a'],
            current_frame: 0,
            frame_time: 0.1,
            timer: 0.0,
            size: 1.0,
        }
    ];

    render_maze(&mut framebuffer, &maze, block, offset_x, offset_y);
    
    window.set_target_fps(15);

    enum GameState {
    StartMenu,
    Playing,
    Paused,
}

let mut state = GameState::StartMenu;

while !window.window_should_close() {
    let dt = window.get_frame_time();

    match state {
        GameState::StartMenu => {
            let mut d = window.begin_drawing(&raylib_thread);
            d.clear_background(Color::BLACK);
            d.draw_text("Presiona ENTER para iniciar", 100, 200, 30, Color::WHITE);
            if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                state = GameState::Playing;
            }
        }
            GameState::Playing => {
                for s in sprites_list.iter_mut() { s.update(dt); }
                process_input(&window, &mut player, &maze, dt);
                framebuffer.clear();
                let zbuf = render3d(&mut framebuffer, &maze, &player, block as usize, offset_x, offset_y, &texman);
                draw_sprites(&mut framebuffer, &player, &sprites_list, block as usize, offset_x, offset_y, &texman, &zbuf);

                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::SKYBLUE);
                framebuffer.draw_maze(&mut d, &raylib_thread);
                let px = (offset_x as f32 + player.pos.x * block as f32) as i32;
                let py = (offset_y as f32 + player.pos.y * block as f32) as i32;
                framebuffer.draw_player(px, py);
                draw_minimap(&mut d, &mut framebuffer, &maze, &player, &sprites_list, 10, 8, block as usize, offset_x, offset_y);
                d.draw_fps(d.get_screen_width() - 100, 10);

                if d.is_key_pressed(KeyboardKey::KEY_P) { state = GameState::Paused; }
            }

            GameState::Paused => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::DARKGRAY);
                d.draw_text("Juego en PAUSA", 150, 200, 40, Color::WHITE);
                d.draw_text("Presiona ENTER para reanudar", 100, 300, 30, Color::WHITE);
                if d.is_key_pressed(KeyboardKey::KEY_ENTER) { state = GameState::Playing; }
            }
        }
    }
}
