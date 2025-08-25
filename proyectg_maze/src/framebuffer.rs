// src/framebuffer.rs

use raylib::prelude::*;
use crate::maze::Maze;


pub struct Framebuffer {
    buffer: Vec<Color>,
    width: u32,
    height: u32,
    current_color: Color,
    background_color: Color,
}

impl Framebuffer {
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    // Creacion del framebuffer
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "Tamaño de Framebuffer invalido, debe ser mayor de 0");
        let background = Color::BLACK;
        let len = (width as usize)
            .checked_mul(height as usize)
            .expect("Framebuffer demasiado grande");
        Self {
            buffer: vec![background; len],
            width,
            height,
            current_color: Color::YELLOW,
            background_color: background,
        }
    }

    // Redimensionar el framebuffer
    pub fn resize(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "Tamaño de Framebuffer invalido, debe ser mayor de 0");
        self.width = width;
        self.height = height;
        let len = (width as usize)
            .checked_mul(height as usize)
            .expect("Framebuffer demasiado grande");
        self.buffer.clear();
        self.buffer.resize(len, self.background_color);
    }

    // Definir el color de fondo
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    // Limpiar el framebuffer
    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
    }

    // Definir el color actual
    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // Escribir un píxel en el framebuffer
    fn write_pixel(&mut self, x: u32, y: u32, color: Color) {
        let idx = (y * self.width + x) as usize;
        self.buffer[idx] = color;
    }

    // Establecer un píxel en el framebuffer
    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.write_pixel(x, y, self.current_color);
        }
    }

    // Establecer un píxel con coordenadas i32
    pub fn set_pixel_i32(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let idx = (y as u32 * self.width + x as u32) as usize;
            self.buffer[idx] = color;
        }
    }

    // Dibujar el laberinto
    pub fn draw_maze(&self, window: &mut RaylibDrawHandle, _: &RaylibThread) {
        for y in 0..self.height {
            let base = (y * self.width) as usize;
            for x in 0..self.width {
                let color = self.buffer[base + x as usize];
                window.draw_pixel(x as i32, y as i32, color);
            }
        }
    }

    // Dibujar el jugador
    pub fn draw_player(&mut self, x: i32, y: i32) {
        self.set_current_color(Color::RED);
        for dx in 0..5 {
            for dy in 0..5 {
                self.set_pixel((x + dx) as u32, (y + dy) as u32);
            }
        }
    }

    // Establecer un píxel grueso en el framebuffer
    pub fn set_thick_pixel(&mut self, x: u32, y: u32, size: u32, color: Color) {
        for dx in 0..size {
            for dy in 0..size {
                let nx = x + dx;
                let ny = y + dy;

                if nx < self.width && ny < self.height {
                    let index = (ny as u32 * self.width + nx as u32) as usize;
                    self.buffer[index] = color;
                }
            }
        }
    } 
}

// codigo suelto del framebuffer
// Determina el color a partir del símbolo
pub fn symbol_to_color(c: char) -> Color {
    match c {
        '+' | '-' | '|' | 'g' => Color::GREEN,
        ' ' => Color::DARKBROWN,
        'h' => Color::ORANGERED,
        _ => Color::BLACK,
    }
}

// Calcular el tamaño del bloque y los offsets
pub fn calc_block_size_offset(
    maze: &Maze,
    win_w: u32,
    win_h: u32,
) -> (u32, i32, i32) {
    let maze_w = maze[0].len() as u32;
    let maze_h = maze.len() as u32;

    let mut block = (win_w / maze_w).min(win_h / maze_h);
    if block == 0 {
        block = 1;
    }

    
    // definicion de funciones total_* y conversion de tamaño de datos u32 a i32
    let total_w = (block * maze_w) as i32;
    let total_h = (block * maze_h) as i32;

    let ww = win_w as i32;
    let wh = win_h as i32;

    // Calcula cuanto espacio sobra y lo divide en 2 para centrar
    let offset_x = ((ww - total_w) / 2).max(0);
    let offset_y = ((wh - total_h) / 2).max(0);

    (block, offset_x, offset_y)
}
