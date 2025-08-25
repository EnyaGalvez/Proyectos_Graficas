// src/controller.rs

use std::f32::consts::{FRAC_PI_2, PI};
use raylib::prelude::*;
use raylib::consts::KeyboardKey;

use crate::maze::{Maze, is_wall};
use crate::player::Player;

fn wrap_angle(mut a: f32) -> f32 { 
    let two_pi: f32 = 2.0_f32 * PI;
    while a >  PI { a -= two_pi; }
    while a < -PI { a += two_pi; }
    a
}

pub fn cell_is_free(maze: &Maze, x: isize, y: isize) -> bool {
    if x < 0 || y < 0 { return false; }
    let (ux, uy) = (x as usize, y as usize);
    if uy >= maze.len() { return false; }
    if ux >= maze[uy].len() { return false; }
    !is_wall(maze[uy][ux])
}

pub fn process_input(rl: &RaylibHandle, player: &mut Player, maze: &Maze, dt: f32) {
    // control de velocidades de movimiento y rotacion
    let move_speed = 3.0; // celdas / segundo
    let rot_speed  = 2.5; // radianes / segundo
    let mouse_sens = 0.0025; // radianes / segundo

    // Rotacion teclado (izquierda y derecha)
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= rot_speed * dt; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += rot_speed * dt; }

    // Rotacion con mouse (solo eje X)
    let md = rl.get_mouse_delta();
    if md.x != 0.0 {
        player.a += md.x * mouse_sens;
    }

    player.a = wrap_angle(player.a);

    // Direcciones: adelante/atras y strafe
    let mut forward = 0.0;
    let mut strafe  = 0.0;

    // Movimiento del jugador: key_up o W = adelante, key_down o S = atras, D = derecha y A = izquierda
    if rl.is_key_down(KeyboardKey::KEY_UP)   || rl.is_key_down(KeyboardKey::KEY_W) { forward += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) { forward -= 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_D) { strafe += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_A) { strafe -= 1.0; }

    if forward != 0.0 || strafe != 0.0 {
        // Vector de movimiento en coordenadas de celdas
        let dir_x = player.a.cos();
        let dir_y = player.a.sin();
        let right_x = (player.a + FRAC_PI_2).cos();
        let right_y = (player.a + FRAC_PI_2).sin();

        let step = move_speed * dt;
        let dx = (dir_x * forward + right_x * strafe) * step;
        let dy = (dir_y * forward + right_y * strafe) * step;

        // Control de colision con paredes, permite que el jugador se deslice en las paredes
        let next_x = player.pos.x + dx;
        let next_y = player.pos.y + dy;

        let cx = next_x.floor() as isize;
        let cy = player.pos.y.floor() as isize;
        if cell_is_free(maze, cx, cy) {
            player.pos.x = next_x;
        }

        let cx2 = player.pos.x.floor() as isize;
        let cy2 = next_y.floor() as isize;
        if cell_is_free(maze, cx2, cy2) {
            player.pos.y = next_y;
        }
    }
}