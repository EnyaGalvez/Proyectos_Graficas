// src/render3d.rs
use raylib::color::Color;

use crate::framebuffer::{Framebuffer, symbol_to_color};
use crate::player::Player;
use crate::caster::cast_ray;
use crate::maze::Maze;
use crate::textures::TextureManager;

// Funcion para crear efecto de vista nublada a lo lejos
fn shade(color: Color, factor: f32) -> Color {
    let f = factor.clamp(0.0, 1.0);
    Color {
        r: (color.r as f32 * f) as u8,
        g: (color.g as f32 * f) as u8,
        b: (color.b as f32 * f) as u8,
        a: color.a,
    }
}

pub fn render3d(
    framebuffer: &mut Framebuffer, 
    maze: &Maze, 
    player: &Player, 
    block_size: usize, 
    offset_x: i32, 
    offset_y: i32,
    tex: &TextureManager
) -> Vec<f32> {
    // ancho y alto
    let w = framebuffer.width() as f32;
    let h = framebuffer.height() as f32;

    // medio ancho y medio alto
    let hw = w / 2.0;
    let hh = h / 2.0;

    let num_rays = framebuffer.width(); // numero de rayos = ancho del framebuffer
    let mut zbuffer = vec![f32::INFINITY; num_rays as usize];

    // Colores base para el cielo y el suelo
    let sky_base = Color::SKYBLUE;
    let floor_char = ' ';
    let (tw, th) = tex.get_image_size(floor_char);

    // angulo de visión horizontal
    let fov = 1.047; // -60 grados
    let b = block_size as f32;

    // posicion del jugador en pixeles
    let start_x = offset_x as f32 + player.pos.x * b;
    let start_y = offset_y as f32 + player.pos.y * b;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // rayo actual dividido en rayos totales
        let a = player.a - (fov / 2.0) + (fov * current_ray);

        if let Some(intersect) = cast_ray(
            framebuffer, &maze, player, a, block_size, offset_x, offset_y, false
        ) {
            let distance = intersect.distance.max(1.0);
            zbuffer[i as usize] = distance;
           
            // Tamaño de las estacas
            let dpp = (w / 2.0) / (fov / 2.0).tan();
            let stake_height = (block_size as f32 / distance) * dpp;

            let stake_top = (hh - (stake_height / 2.0).max(0.0)) as usize;
            let stake_bottom = (hh + (stake_height / 2.0).min(h)) as usize;

            // color del cielo
            for y in 0..stake_top {
                framebuffer.set_pixel_i32(i as i32, y as i32, sky_base);
            }
            
            // Textura de paredes
            let wall_ch = intersect.impact;
            let (tw_wall, th_wall) = tex.get_image_size(wall_ch);

            // Coordenadas locales dentro del tile donde impactó
            let local_x = (intersect.hit_x - offset_x as f32).rem_euclid(b);
            let local_y = (intersect.hit_y - offset_y as f32).rem_euclid(b);

            // u en [0..1] segun cara
            let mut u = if intersect.vertical { local_y / b } else { local_x / b };
            // Voltear para mantener orientación consistente
            if intersect.vertical {
                if a.cos() < 0.0 { u = 1.0 - u; }
            } else {
                if a.sin() > 0.0 { u = 1.0 - u; }
            }
            let tx = (u * tw_wall as f32).clamp(0.0, (tw_wall - 1) as f32) as u32;

            let shade_factor = (1.0 / (1.0 + 0.007 * distance)).clamp(0.5, 1.0);

            for y in stake_top..stake_bottom {
                let rel = ((y as f32) - (stake_top as f32)) / stake_height.max(1.0);
                let ty = (rel * th_wall as f32).clamp(0.0, (th_wall - 1) as f32) as u32;

                let texel = tex.get_pixel_color(wall_ch, tx, ty);
                let color = shade(texel, shade_factor);

                framebuffer.set_pixel_i32(i as i32, y as i32, color);
            }

            // Textura del suelo
            for y in stake_bottom..(h as usize) {
                let yf = y as f32;

                // Evitar división por 0 cerca del horizonte
                if yf <= hh + 0.5 { 
                    framebuffer.set_pixel_i32(i as i32, y as i32, sky_base);
                    continue;
                }
                // Distancia aproximada al punto del piso
                let dist_y = (b * dpp) / (2.0 * (yf - hh));

                // Punto del mundo a esa distancia sobre el rayo de este píxel
                let fx = start_x + dist_y * a.cos();
                let fy = start_y + dist_y * a.sin();

                // Coordenada local dentro del tile (0..block_size)
                let local_x = (fx - offset_x as f32).rem_euclid(b);
                let local_y = (fy - offset_y as f32).rem_euclid(b);

                // Llevar a coordenadas de textura (0..tw/th)
                let tx = ((local_x / b) * tw as f32) as u32;
                let ty = ((local_y / b) * th as f32) as u32;

                // Muestrear textura y aplicar un sombreado suave por profundidad visual
                let floor_color = tex.get_pixel_color(floor_char, tx, ty);

                framebuffer.set_pixel_i32(i as i32, y as i32, floor_color);
            }

        } else { // Si el rayo no golpea nada, la region se mantiene negra
            for y in 0..(h as usize) {
                let color = if (y as f32) < hh { sky_base } else {
                    tex.get_pixel_color(floor_char, 0, 0)
                    };
                framebuffer.set_pixel_i32(i as i32, y as i32, color);
            }
        }
    }

    return zbuffer;
}