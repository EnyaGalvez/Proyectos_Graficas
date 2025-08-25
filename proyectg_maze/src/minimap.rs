// src/minimap.rs
use raylib::prelude::*;

use crate::framebuffer::{symbol_to_color, Framebuffer};
use crate::maze::Maze;
use crate::player::Player;
use crate::sprites::Sprite;
use crate::caster::cast_ray;

// Establece el minimapa en la esquina superior izquierda
pub fn draw_minimap(
    d: &mut RaylibDrawHandle,
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    sprites: &[Sprite],
    tile_px: i32,
    margin: i32,
    block_size: usize,
    offset_x: i32,
    offset_y: i32,
) {
    let tile_px = tile_px.max(2);
    let x0 = margin;
    let y0 = margin;

    let map_w = (maze[0].len() as i32) * tile_px;
    let map_h = (maze.len() as i32) * tile_px;

    // fondo y borde
    d.draw_rectangle(x0 - 2, y0 - 2, map_w + 4, map_h + 4, Color::new(0, 0, 0, 160));
    d.draw_rectangle_lines(x0 - 2, y0 - 2, map_w + 4, map_h + 4, Color::GREEN);

    // celdas implementando symbol_to_color
    for (ry, row) in maze.iter().enumerate() {
        for (rx, &cell) in row.iter().enumerate() {
            let c = symbol_to_color(cell);
            d.draw_rectangle(
                x0 + (rx as i32) * tile_px,
                y0 + (ry as i32) * tile_px,
                tile_px,
                tile_px,
                c,
            );
        }
    }

    // jugador
    let px = x0 as f32 + player.pos.x * tile_px as f32;
    let py = y0 as f32 + player.pos.y * tile_px as f32;
    d.draw_circle(px as i32, py as i32, ((tile_px as f32) * 0.45).max(2.0), Color::INDIGO);

    // FOV con rayos usando cast_ray
    let fov = 1.047; // 60
    let rays = 60;   // numero de rayos
    let b = block_size as f32;

    for i in 0..rays {
        let t = i as f32 / (rays - 1) as f32;
        let a = player.a - (fov * 0.5) + (fov * t);

        if let Some(hit) = cast_ray(
            framebuffer,
            maze,
            player,
            a,
            block_size,
            offset_x,
            offset_y,
            false,
        ) {
            // punto de impacto en coordenadas de CELDA (mundo 2D):
            let mx = (hit.hit_x - offset_x as f32) / b;
            let my = (hit.hit_y - offset_y as f32) / b;

            // llevar a coords del minimapa
            let ex = x0 as f32 + mx * tile_px as f32;
            let ey = y0 as f32 + my * tile_px as f32;

            // linea del jugador hacia el impacto
            d.draw_line(px as i32, py as i32, ex as i32, ey as i32, Color::WHITE);
        } else {
            // si no choca (salió del mapa), traza un segmento corto en dirección
            let ex = px + (tile_px as f32) * 2.0 * a.cos();
            let ey = py + (tile_px as f32) * 2.0 * a.sin();
            d.draw_line(px as i32, py as i32, ex as i32, ey as i32, Color::WHITE);
        }
    }

    // sprites
    let r_sprite = ((tile_px as f32) * 0.35).max(2.0);
    for s in sprites {
        // posicion del sprite en minimapa
        let sx = x0 as f32 + s.pos.x * tile_px as f32;
        let sy = y0 as f32 + s.pos.y * tile_px as f32;

        let mut col = symbol_to_color(s.current_tex_key());

        d.draw_circle(sx as i32, sy as i32, r_sprite, col);
    }

    // línea corta de orientación del jugador
    let len = tile_px as f32 * 0.9;
    let dx = px + len * player.a.cos();
    let dy = py + len * player.a.sin();
    d.draw_line(px as i32, py as i32, dx as i32, dy as i32, Color::SKYBLUE);
}