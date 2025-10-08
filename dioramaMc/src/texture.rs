// src/texture.rs
use image::io::Reader as ImageReader;
use nalgebra_glm::Vec3;
use crate::color::Color;

const TILE: usize = 8;

const TEX_MAX_W: u32 = 512;
const TEX_MAX_H: u32 = 512;

#[derive(Debug, Clone)]
pub struct Texture {
    pub w: usize,
    pub h: usize,
    pw: usize,
    ph: usize,
    tiles_x: usize,
    tiles_y: usize,
    pub data_swz: Vec<Color>,
}

impl Texture {
    pub fn from_file(path: &str) -> Self {
        let img0 = ImageReader::open(path)
            .expect("No se pudo abrir la textura")
            .decode()
            .expect("Error al decodificar imagen")
            .to_rgb8();

        let (w0, h0) = img0.dimensions();
        let scale = f32::min(TEX_MAX_W as f32 / w0 as f32, TEX_MAX_H as f32 / h0 as f32);
        let img = if scale < 1.0 {
            let tw = (w0 as f32 * scale).floor().max(1.0) as u32;
            let th = (h0 as f32 * scale).floor().max(1.0) as u32;
            image::imageops::resize(&img0, tw, th, image::imageops::FilterType::Triangle)
        } else {
            img0
        };

        let (w_u32, h_u32) = img.dimensions();
        let (w, h) = (w_u32 as usize, h_u32 as usize);

        let pw = ((w + TILE - 1) / TILE) * TILE;
        let ph = ((h + TILE - 1) / TILE) * TILE;

        let tiles_x = pw / TILE;
        let tiles_y = ph / TILE;

        let mut row_major: Vec<Color> = Vec::with_capacity(w * h);
        for p in img.pixels() {
            row_major.push(Color::new(p[0], p[1], p[2]));
        }

        let mut data_swz = vec![Color::new(0, 0, 0); pw * ph];

        let clamp = |x: isize, a: usize| -> usize {
            if x < 0 { 0 } else if (x as usize) >= a { a - 1 } else { x as usize }
        };

        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_base = (ty * tiles_x + tx) * (TILE * TILE);

                for iy in 0..TILE {
                    for ix in 0..TILE {
                        let x = tx * TILE + ix;
                        let y = ty * TILE + iy;

                        let src_x = clamp(x as isize, w);
                        let src_y = clamp(y as isize, h);

                        let src_idx = src_y * w + src_x;
                        let dst_idx = tile_base + iy * TILE + ix;

                        data_swz[dst_idx] = row_major[src_idx];
                    }
                }
            }
        }

        Texture {
            w, h, pw, ph, tiles_x, tiles_y, data_swz
        }
    }

    #[inline]
    fn idx_swizzled(&self, x: usize, y: usize) -> usize {
        let tx = x / TILE;
        let ty = y / TILE;
        let ix = x % TILE;
        let iy = y % TILE;

        let tile_base = (ty * self.tiles_x + tx) * (TILE * TILE);
        tile_base + iy * TILE + ix
    }

    #[inline]
    fn pixel_swizzled(&self, x: usize, y: usize) -> Color {
        let cx = x.min(self.w - 1);
        let cy = y.min(self.h - 1);
        let idx = self.idx_swizzled(cx, cy);
        self.data_swz[idx]
    }

    #[inline]
    fn wrap_uv(u: f32, v: f32) -> (f32, f32) {
        let uu = u.fract().rem_euclid(1.0);
        let vv = v.fract().rem_euclid(1.0);
        (uu, vv)
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let (u, v) = Self::wrap_uv(u, v);
        let x = (u * (self.w as f32 - 1.0)).round() as usize;
        let y = ((1.0 - v) * (self.h as f32 - 1.0)).round() as usize; // origen arriba-izq
        self.pixel_swizzled(x, y)
    }

     pub fn sample_tiled_uv(&self, u: f32, v: f32, tiling_u: f32, tiling_v: f32) -> Color {
        self.sample(u * tiling_u, v * tiling_v)
    }

    pub fn sample_normal_tangent_uv(&self, u: f32, v: f32, tiling_u: f32, tiling_v: f32) -> Vec3 {
        let c = self.sample_tiled_uv(u, v, tiling_u, tiling_v);
        let nx = (c.r as f32 / 255.0) * 2.0 - 1.0;
        let ny = (c.g as f32 / 255.0) * 2.0 - 1.0;
        let nz = (c.b as f32 / 255.0) * 2.0 - 1.0;
        nalgebra_glm::normalize(&Vec3::new(nx, ny, nz))
    }
}