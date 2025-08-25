// src/caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::maze::{is_wall, Maze};

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32,
    pub hit_y: f32,
    pub vertical: bool
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer, 
    maze: &Maze, 
    player: &Player,
    a: f32, 
    block_size: usize, 
    offset_x: i32, 
    offset_y: i32,
    draw_line: bool
) -> Option<Intersect> {
    let mut d = 0.0;
    framebuffer.set_current_color(Color::PINK);

    let b = block_size as f32;

    let start_x = offset_x as f32 + player.pos.x * b;
    let start_y = offset_y as f32 + player.pos.y * b;

    loop {
        // posición flotante del rayo
        let xf = start_x + d * a.cos();
        let yf = start_y + d * a.sin();

        // enteros para indexar
        let x = xf as i32;
        let y = yf as i32;
        
        if x < 0 || y < 0 { return None; }

        let cell_x = ((x - offset_x) as f32 / b).floor() as isize;
        let cell_y = ((y - offset_y) as f32 / b).floor() as isize;

        // límites del laberinto
        if cell_y < 0 || cell_x < 0 { return None; }
        let uy = cell_y as usize;
        let ux = cell_x as usize;
        if uy >= maze.len() || ux >= maze[uy].len() { return None; }

        // gestion de colisión con muros en vertical y horizontal
        if is_wall(maze[uy][ux]) {
            let d_prev = if d > 0.5 { d - 0.5 } else { d };
            let x_prev = (start_x + d_prev * a.cos()) as i32;
            let y_prev = (start_y + d_prev * a.sin()) as i32;

            let cell_x_prev = ((x_prev - offset_x) as f32 / b).floor() as isize;
            let cell_y_prev = ((y_prev - offset_y) as f32 / b).floor() as isize;

            let vertical = if ux as isize != cell_x_prev && uy as isize == cell_y_prev {
                true  
            } else if uy as isize != cell_y_prev && ux as isize == cell_x_prev {
                false
            } else {
                a.cos().abs() > a.sin().abs()
            };

            return Some(Intersect { 
                distance: d, 
                impact: maze[uy][ux],
                hit_x: xf,
                hit_y: yf,
                vertical,
            });
        }

        // dibuja si draw_line es verdadero
        if draw_line {
            framebuffer.set_pixel(x as u32, y as u32);
        }

        d += 1.0;
    }
}