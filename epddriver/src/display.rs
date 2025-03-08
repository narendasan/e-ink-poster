use consts::{HEIGHT, WIDTH};
use hal;
use hal::blocking::delay::DelayMs;
trait ColorSpace {}

struct Display<'a> {
    buffer: &'a mut [u8; HEIGHT as usize * WIDTH as usize],
    driver: Driver,
}

impl Display {
    pub fn new(height: u32, width: u32) -> Self {
        let buffer = vec![0; (height * width) as usize];
        Display {
            buffer: DisplayBuffer {
                height,
                width,
                buffer,
            },
            driver: Driver::new(),
        }
    }

    pub fn dither(&mut self) {
        // Implement dithering algorithm here
    }

    pub fn apply_colormap(&mut self, colorspace: ColorSpace) {
        for color in &mut self.buffer.buffer {
            *color = colorspace.apply(*color);
        }
    }

    pub fn render(&mut self, delay: &mut DelayMs<u32>) {
        self.driver.write(&self.buffer);
    }

    pub fn clear(&mut self) {
        self.driver.write(&[0x11] * self.buffer.buffer.len());
    }
}
