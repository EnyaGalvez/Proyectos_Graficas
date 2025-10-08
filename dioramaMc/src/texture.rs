// src/texture.rs
use image::io::Reader as ImageReader;
use nalgebra_glm::Vec3;
use crate::color::Color;

const TILE: usize = 8;

const TEX_MAX_W: u32 = 512;
const TEX_MAX_H: u32 = 512;

#[derive(Debug, Clone)]
struct Mip {
    w: usize,
    h: usize,
    pw: usize,
    ph: usize,
    tiles_x: usize,
    tiles_y: usize,
    data_swz: Vec<Color> 

}

#[derive(Debug, Clone)]
pub struct Texture {
    pub w: usize,
    pub h: usize,
    pw: usize,
    ph: usize,
    tiles_x: usize,
    tiles_y: usize,
    pub data_swz: Vec<Color>,
    mips: Vec<Mip>
}

#[derive(Debug, Clone, Copy)]
pub struct TextureOptions {
    pub max_w: u32,
    pub max_h: u32,
    pub tile: usize,
    pub generate_mips: bool,
    pub max_mips: usize,
    pub min_mip_area: usize,
}

impl Default for TextureOptions {
    fn default() -> Self {
        TextureOptions {
            max_w: TEX_MAX_W,
            max_h: TEX_MAX_H,
            tile: TILE,
            generate_mips: true,
            max_mips: 8,
            min_mip_area: 64,
        }
    }
}

impl Texture {
    pub fn from_file_with(path: &str, opt: TextureOptions) -> Self {
        let img0 = ImageReader::open(path)
            .expect("No se pudo abrir la textura")
            .decode()
            .expect("Error al decodificar imagen")
            .to_rgb8();

        let (w0, h0) = img0.dimensions();
        let scale = f32::min(opt.max_w as f32 / w0 as f32, opt.max_h as f32 / h0 as f32);
        let img = if scale < 1.0 {
            let tw = (w0 as f32 * scale).floor().max(1.0) as u32;
            let th = (h0 as f32 * scale).floor().max(1.0) as u32;
            image::imageops::resize(&img0, tw, th, image::imageops::FilterType::Triangle)
        } else {
            img0
        };

        let (w_u32, h_u32) = img.dimensions();
        let (w, h) = (w_u32 as usize, h_u32 as usize);

        let mut row0: Vec<Color> = Vec::with_capacity(w * h);
        for p in img.pixels() {
            row0.push(Color::new(p[0], p[1], p[2]));
        }

        let (pw, ph, tiles_x, tiles_y, base_swz) = Self::swizzle_from_row_major(&row0, w, h, opt.tile);

        let mips = if opt.generate_mips {
            Self::build_mips_swizzled(row0, w, h, opt)
        } else {
            Vec::new()
        };

        Texture { w, h, pw, ph, tiles_x, tiles_y, data_swz: base_swz, mips }
    
    }

    pub fn from_file(path: &str) -> Self {
        Self::from_file_with(path, TextureOptions::default())
    }

    fn swizzle_from_row_major(row: &[Color], w: usize, h: usize, tile: usize) -> (usize, usize, usize, usize, Vec<Color>) {
        
        let pw = ((w + tile - 1) / tile) * tile;
        let ph = ((h + tile - 1) / tile) * tile;
        let tiles_x = pw / tile;
        let tiles_y = ph / tile;

        let mut data_swz = vec![Color::new(0, 0, 0); pw * ph];

        let clamp = |x: isize, a: usize| -> usize {
            if x < 0 { 0 } else if (x as usize) >= a { a - 1 } else { x as usize }
        };


        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_base = (ty * tiles_x + tx) * (tile * tile);

                for iy in 0..tile {
                    for ix in 0..tile {
                        let x = tx * tile + ix;
                        let y = ty * tile + iy;

                        let src_x = clamp(x as isize, w);
                        let src_y = clamp(y as isize, h);

                        let src_idx = src_y * w + src_x;
                        let dst_idx = tile_base + iy * tile + ix;

                        data_swz[dst_idx] = row[src_idx];
                    }
                }
            }
        }
        (pw, ph, tiles_x, tiles_y, data_swz)
    }

    fn build_mips_swizzled(mut row_prev: Vec<Color>, mut w_prev: usize, mut h_prev: usize, opts: TextureOptions) -> Vec<Mip> {
        let mut out = Vec::new();
        let mut levels = 0usize;

        while w_prev > 1 && h_prev > 1 && levels < opts.max_mips {
            let nw = (w_prev / 2).max(1);
            let nh = (h_prev / 2).max(1);

            if nw * nh < opts.min_mip_area {
                break;
            }

            let mut row_next = Vec::with_capacity(nw * nh);
            for y in 0..nh {
                for x in 0..nw {
                    let ix = x * 2;
                    let iy = y * 2;
                    
                    let get = |xx: usize, yy: usize| -> Color {
                        row_prev[yy * w_prev + xx]
                    };

                    let x1 = (ix + 1).min(w_prev - 1);
                    let y1 = (iy + 1).min(h_prev - 1);

                    let c0 = get(ix,  iy);
                    let c1 = get(x1, iy);
                    let c2 = get(ix,  y1);
                    let c3 = get(x1, y1);

                    let avg = Color::new(
                        ((c0.r as u16 + c1.r as u16 + c2.r as u16 + c3.r as u16) / 4) as u8,
                        ((c0.g as u16 + c1.g as u16 + c2.g as u16 + c3.g as u16) / 4) as u8,
                        ((c0.b as u16 + c1.b as u16 + c2.b as u16 + c3.b as u16) / 4) as u8,
                    );
                    row_next.push(avg);
                }
            }

            // Swizzle del mip
            let (pw, ph, tiles_x, tiles_y, data_swz) = Self::swizzle_from_row_major(&row_next, nw, nh, opts.tile);
            out.push(Mip { w: nw, h: nh, pw, ph, tiles_x, tiles_y, data_swz });

            // Siguiente iteración
            row_prev = row_next;
            w_prev = nw;
            h_prev = nh;
            levels += 1;
        }

        out
    }

    #[inline]
    fn idx_swizzled(pw: usize, tiles_x: usize, tile: usize, x: usize, y: usize) -> usize {
        let tx = x / tile;
        let ty = y / tile;
        let ix = x % tile;
        let iy = y % tile;
        (ty * tiles_x + tx) * ( tile * tile) + iy * tile + ix
    }

    #[inline]
    fn pixel_swizzled_at(data: &[Color], pw: usize, tiles_x: usize, tile: usize, w: usize, h: usize, x: usize, y: usize) -> Color {
        let cx = x.min(w - 1);
        let cy = y.min(h - 1);
        let idx = Self::idx_swizzled(pw, tiles_x, tile, cx, cy);
        data[idx]
    }

    #[inline]
    fn pixel_swizzled(&self, x: usize, y: usize) -> Color {
        Self::pixel_swizzled_at(&self.data_swz, self.pw, self.tiles_x, TILE, self.w, self.h, x, y)
    }


    #[inline]
    fn pixel_swizzled_mip(m: &Mip, x: usize, y: usize) -> Color {
        Self::pixel_swizzled_at(&m.data_swz, m.pw, m.tiles_x, TILE, m.w, m.h, x, y)
    }

    #[inline]
    fn wrap_uv(u: f32, v: f32) -> (f32, f32) {
        (u.fract().rem_euclid(1.0), v.fract().rem_euclid(1.0))
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let (u, v) = Self::wrap_uv(u, v);
        let x = (u * (self.w as f32 - 1.0)).round() as usize;
        let y = ((1.0 - v) * (self.h as f32 - 1.0)).round() as usize;
        self.pixel_swizzled(x, y)
    }

     pub fn sample_tiled_uv(&self, u: f32, v: f32, tiling_u: f32, tiling_v: f32) -> Color {
        self.sample(u * tiling_u, v * tiling_v)
    }

    fn pick_mip_index(&self, tiling_u: f32, tiling_v: f32) -> Option<usize> {
        if self.mips.is_empty() { return None; }
        let s = tiling_u.abs().max(tiling_v.abs()).max(1.0);
        let lod = s.log2().floor() as i32;
        if lod <= 0 { return None; }
        let idx = (lod as usize - 1).min(self.mips.len() - 1);
        Some(idx)
    }

    pub fn sample_tiled_mip(&self, u: f32, v: f32, tiling_u: f32, tiling_v: f32) -> Color {
        if let Some(mi) = self.pick_mip_index(tiling_u, tiling_v) {
            let m = &self.mips[mi];
            let (u, v) = Self::wrap_uv(u * tiling_u, v * tiling_v);
            let x = (u * (m.w as f32 - 1.0)).round() as usize;
            let y = ((1.0 - v) * (m.h as f32 - 1.0)).round() as usize;
            return Self::pixel_swizzled_mip(m, x, y);
        }
        self.sample_tiled_uv(u, v, tiling_u, tiling_v)
    }

    pub fn sample_normal_tangent_uv(&self, u: f32, v: f32, tu: f32, tv: f32) -> Vec3 {
        let c = self.sample_tiled_uv(u, v, tu, tv);
        let nx = (c.r as f32 / 255.0) * 2.0 - 1.0;
        let ny = (c.g as f32 / 255.0) * 2.0 - 1.0;
        let nz = (c.b as f32 / 255.0) * 2.0 - 1.0;
        nalgebra_glm::normalize(&Vec3::new(nx, ny, nz))
    }

    pub fn sample_normal_tangent_mip_uv(&self, u: f32, v: f32, tu: f32, tv: f32) -> Vec3 {
        let c = self.sample_tiled_mip(u, v, tu, tv);
        let nx = (c.r as f32 / 255.0) * 2.0 - 1.0;
        let ny = (c.g as f32 / 255.0) * 2.0 - 1.0;
        let nz = (c.b as f32 / 255.0) * 2.0 - 1.0;
        nalgebra_glm::normalize(&Vec3::new(nx, ny, nz))
    }
}