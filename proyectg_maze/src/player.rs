// src/player.rs
use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2, // definicion de player como vector
    pub a: f32, // angulo de vista
}