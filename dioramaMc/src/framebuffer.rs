// src/framebuffer.rs

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x3377ff,
            current_color: 0xFFFFFF,
        }
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    #[inline]
    pub fn row_mut(&mut self, y: usize) -> &mut [u32] {
        let w = self.width;
        let start = y * w;
        &mut self.buffer[start..start + w]
    }

    #[inline]
    pub fn write_row(&mut self, y: usize, row: &[u32]) {
        debug_assert_eq!(row.len(), self.width);
        let dst = self.row_mut(y);
        dst.copy_from_slice(row);
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    #[inline]
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn fill(&mut self, color: u32) {
        self.buffer.fill(color);
    }

    pub fn clear(&mut self) {
        self.fill(self.background_color);
    }
}