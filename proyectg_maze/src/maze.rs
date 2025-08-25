// src/maze.rs

use crate::framebuffer::symbol_to_color;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::framebuffer::Framebuffer;

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).expect("Error: Could not open maze file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            line.unwrap()
                .chars()
                .filter(|c| matches!(c, '+' | '-' | '|' | 'g' | ' ' ))
                .collect()
        })
        .collect()
}

pub fn is_wall(c: char) -> bool {
    matches!(c, '+' | '-' | '|' | 'g')
}

pub fn find_first_free_cell(maze: &Maze) -> Option<(usize, usize)> {
    for (y, row) in maze.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == ' ' {
                return Some((x, y));
            }
        }
    }
    None
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: u32,
    offset_x: i32,
    offset_y: i32,

) {
    let b = block_size as i32;
    
    for (y, row) in maze.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let color = symbol_to_color(cell);

            let px = (offset_x + (x as i32) * b) as u32;
            let py = (offset_y + (y as i32) * b) as u32;
            framebuffer.set_thick_pixel(px, py, block_size, color);
        }
    }
}