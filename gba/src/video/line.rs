pub struct LineBuffer {
    // 240 pixels for each background (BG0-3)
    pixels: [[u16; 240]; 4],
}

impl Default for LineBuffer {
    fn default() -> Self {
        LineBuffer {
            pixels: [[0; 240]; 4],
        }
    }
}

impl LineBuffer {
    pub fn put(&mut self, bg: usize, x: usize, pixel: u16) {
        self.pixels[bg][x] = pixel;
    }

    // pub fn get(&self, bg: usize, x: usize) -> u16 {
    //     self.pixels[bg][x]
    // }

    pub fn bg(&self, bg: usize) -> &[u16; 240] {
        &self.pixels[bg]
    }
}
