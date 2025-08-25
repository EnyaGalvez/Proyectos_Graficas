// src/hud.rs
use raylib::prelude::*;

/// Muestra los FPS en la esquina superior izquierda
pub fn draw_fps_top_left(d: &mut RaylibDrawHandle) {
    let screen_w = d.get_screen_width();
    d.draw_fps(screen_w - 100, 10);
}