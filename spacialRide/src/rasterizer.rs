// src/rasterizer.rs
use nalgebra_glm::Vec3;
use crate::color::Color;
use crate::framebuffer::Framebuffer;

/// Calcula el bounding box de un triángulo
#[inline]
fn bbox3(a: Vec3, b: Vec3, c: Vec3, w: usize, h: usize) -> (usize, usize, usize, usize) {
    let min_x = a.x.min(b.x).min(c.x).floor() as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
    let min_y = a.y.min(b.y).min(c.y).floor() as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil() as i32;
    
    (
        min_x.clamp(0, w as i32 - 1) as usize,
        max_x.clamp(0, w as i32 - 1) as usize,
        min_y.clamp(0, h as i32 - 1) as usize,
        max_y.clamp(0, h as i32 - 1) as usize,
    )
}

/// Calcula coordenadas baricéntricas
#[inline]
fn barycentric(a: Vec3, b: Vec3, c: Vec3, px: f32, py: f32) -> (f32, f32, f32) {
    let v0x = b.x - a.x;
    let v0y = b.y - a.y;
    let v1x = c.x - a.x;
    let v1y = c.y - a.y;
    let v2x = px - a.x;
    let v2y = py - a.y;
    
    let denom = v0x * v1y - v1x * v0y;
    
    if denom.abs() < 1e-8 {
        return (-1.0, -1.0, -1.0);
    }
    
    let u = (v2x * v1y - v1x * v2y) / denom;
    let v = (v0x * v2y - v2x * v0y) / denom;
    let w = 1.0 - u - v;
    
    (u, v, w)
}

/// Z-buffer para depth testing
pub struct ZBuffer {
    buffer: Vec<f32>,
    width: usize,
    height: usize,
}

impl ZBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: vec![f32::INFINITY; width * height],
            width,
            height,
        }
    }
    
    pub fn clear(&mut self) {
        for z in self.buffer.iter_mut() {
            *z = f32::INFINITY;
        }
    }
    
    pub fn test_and_set(&mut self, x: usize, y: usize, z: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let idx = y * self.width + x;
        
        if z < self.buffer[idx] {
            self.buffer[idx] = z;
            true
        } else {
            false
        }
    }
}

/// Dibuja un triángulo relleno usando coordenadas baricéntricas
pub fn draw_triangle(
    framebuffer: &mut Framebuffer,
    zbuffer: &mut ZBuffer,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    base_color: Color,
) {
    let (min_x, max_x, min_y, max_y) = bbox3(a, b, c, framebuffer.width, framebuffer.height);
    
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            
            let (u, v, w) = barycentric(a, b, c, px, py);
            
            // Verificar si el punto está dentro del triángulo
            if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                // Interpolar Z (profundidad)
                let z = a.z * w + b.z * u + c.z * v;
                
                // Z-test: solo dibujar si está más cerca
                if zbuffer.test_and_set(x, y, z) {
                    // Aplicar gradiente de color basado en profundidad (opcional)
                    let color = apply_depth_shading(base_color, z, a, b, c);
                    framebuffer.set_pixel(x, y, color);
                }
            }
        }
    }
}

/// Aplica shading basado en profundidad
fn apply_depth_shading(base_color: Color, z: f32, a: Vec3, b: Vec3, c: Vec3) -> Color {
    // Encontrar rango de Z del triángulo
    let tri_zmin = a.z.min(b.z).min(c.z);
    let tri_zmax = a.z.max(b.z).max(c.z);
    
    // Normalizar Z a [0, 1] dentro del triángulo
    let mut t = if (tri_zmax - tri_zmin).abs() > 1e-6 {
        ((z - tri_zmin) / (tri_zmax - tri_zmin)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    
    // Ajustar rango para mejor visualización
    let edge0 = 0.10;
    let edge1 = 0.60;
    t = ((t - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    
    // Aplicar curva
    t = t.powf(0.4);
    
    // Color cercano vs lejano
    let near_color = base_color;
    let far_color = Color::new(102, 153, 153);
    
    // Interpolar
    let r = mix(near_color.r, far_color.r, t);
    let g = mix(near_color.g, far_color.g, t);
    let b = mix(near_color.b, far_color.b, t);
    
    Color::new(r, g, b)
}

#[inline]
fn mix(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}

/// Dibuja línea entre dos puntos (para wireframe)
pub fn draw_line(
    framebuffer: &mut Framebuffer,
    mut x1: i32,
    mut y1: i32,
    x2: i32,
    y2: i32,
    color: Color,
) {
    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = -(y2 - y1).abs();
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        if x1 >= 0 && x1 < framebuffer.width as i32 && y1 >= 0 && y1 < framebuffer.height as i32 {
            framebuffer.set_pixel(x1 as usize, y1 as usize, color);
        }
        
        if x1 == x2 && y1 == y2 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
}

/// Dibuja triángulo en modo wireframe
pub fn draw_triangle_wireframe(
    framebuffer: &mut Framebuffer,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    color: Color,
) {
    draw_line(framebuffer, a.x as i32, a.y as i32, b.x as i32, b.y as i32, color);
    draw_line(framebuffer, b.x as i32, b.y as i32, c.x as i32, c.y as i32, color);
    draw_line(framebuffer, c.x as i32, c.y as i32, a.x as i32, a.y as i32, color);
}