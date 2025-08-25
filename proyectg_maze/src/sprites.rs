// src/sprites.rs
use std::f32::consts::{PI};
use raylib::prelude::*;
use raylib::color::Color;

use crate::player::Player;
use crate::framebuffer::Framebuffer;
use crate::textures::TextureManager;

const FOV: f32 = 1.047; // (60)

pub struct Sprite {
    pub pos: Vector2,
    pub tex_keys: Vec<char>,
    pub current_frame: usize,
    pub frame_time: f32,
    pub timer: f32,
    pub size: f32,
}

impl Sprite {
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.frame_time {
            self.timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.tex_keys.len();
        }
    }

    pub fn current_tex_key(&self) -> char {
        self.tex_keys[self.current_frame]
    }
}

fn shade(color: Color, factor: f32) -> Color {
    let f = factor.clamp(0.0, 1.0);
    Color {
        r: (color.r as f32 * f) as u8,
        g: (color.g as f32 * f) as u8,
        b: (color.b as f32 * f) as u8,
        a: color.a,
    }
}

fn normalize_angle(mut a: f32) -> f32 {
    let two_pi: f32 = 2.0_f32 * PI;
    while a >  PI { a -= two_pi; }
    while a < -PI { a += two_pi; }
    a
}

/// Dibuja todos los sprites con recorte por FOV y oclusión usando z-buffer.
pub fn draw_sprites(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprites: &[Sprite],
    block_size: usize,
    _offset_x: i32,
    _offset_y: i32,
    tex: &TextureManager,
    zbuffer: &[f32],
) {
    let w = framebuffer.width() as f32;
    let h = framebuffer.height() as f32;
    let hw = w * 0.5;
    let hh = h * 0.5;

    // plano de proyección (mismo que en render3d)
    let dpp = (w * 0.5) / (FOV * 0.5).tan();
    let b = block_size as f32;

    // Ordena por distancia descendente (pintar de lejos a cerca)
    let mut order: Vec<(usize, f32)> = sprites.iter().enumerate().map(|(i, s)| {
        let dxp = (s.pos.x - player.pos.x) * b;
        let dyp = (s.pos.y - player.pos.y) * b;
        (i, (dxp*dxp + dyp*dyp).sqrt())
    }).collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (idx, dist) in order {
        let spr = &sprites[idx];

        // vector al sprite (en píxeles del mundo)
        let dxp = (spr.pos.x - player.pos.x) * b;
        let dyp = (spr.pos.y - player.pos.y) * b;

        // ángulo relativo al jugador
        let ang_to_sprite = dyp.atan2(dxp);
        let mut ang = normalize_angle(ang_to_sprite - player.a);

        // si está muy fuera del FOV, lo omitimos
        let extra_margin = 0.1; // pequeño margen
        if ang.abs() > (FOV * 0.5 + extra_margin) {
            continue;
        }

        // posición horizontal en la pantalla (centro del sprite)
        let screen_x = hw * (1.0 + ang / (FOV * 0.5));

        // tamaño proyectado (cuadrado)
        let size_px = ((block_size as f32) / dist.max(1.0)) * dpp * spr.size;
        let half = size_px * 0.5;

        // caja en pantalla
        let mut start_x = (screen_x - half).floor() as i32;
        let mut end_x   = (screen_x + half).ceil() as i32;
        let mut start_y = (hh - half).floor() as i32;
        let mut end_y   = (hh + half).ceil() as i32;

        // recorte a la pantalla
        if end_x < 0 || start_x >= w as i32 { continue; }
        if end_y < 0 || start_y >= h as i32 { continue; }
        start_x = start_x.max(0);
        start_y = start_y.max(0);
        end_x   = end_x.min(w as i32 - 1);
        end_y   = end_y.min(h as i32 - 1);

        let (tw, th) = tex.get_image_size(spr.current_tex_key());
        let shade_factor = (1.0 / (1.0 + 0.007 * dist)).clamp(0.5, 1.0);

        // barrido por columnas con test de profundidad por zbuffer
        for x in start_x..=end_x {
            let col = x as usize;
            // si está detrás de la pared en esta columna, lo saltamos
            if col < zbuffer.len() && dist >= zbuffer[col] {
                continue;
            }

            let u = ((x as f32 - (screen_x - half)) / size_px).clamp(0.0, 1.0);
            let tx = (u * (tw - 1) as f32) as u32;

            for y in start_y..=end_y {
                let v = ((y as f32 - (hh - half)) / size_px).clamp(0.0, 1.0);
                let ty = (v * (th - 1) as f32) as u32;

                let px = tex.get_pixel_color(spr.current_tex_key(), tx, ty);

                let c = shade(px, shade_factor);
                framebuffer.set_pixel_i32(x, y, c);
            }
        }
    }
}