use embedded_graphics_core::{pixelcolor::BinaryColor, prelude::*};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 480;

pub const BYTES_PER_ROW: usize = WIDTH.div_ceil(8);
pub const FRAME_BYTES: usize = BYTES_PER_ROW * HEIGHT;

pub struct FrameBuffer {
    data: [u8; FRAME_BYTES],
}

impl FrameBuffer {
    pub const fn new(fill: u8) -> Self {
        Self {
            data: [fill; FRAME_BYTES],
        }
    }

    pub fn clear(&mut self, fill: u8) {
        self.data.fill(fill);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: BinaryColor) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        let index = y * BYTES_PER_ROW + x / 8;
        let mask = 0x80 >> (x % 8);

        match color {
            BinaryColor::On => {
                self.data[index] &= !mask;
            }
            BinaryColor::Off => {
                self.data[index] |= mask;
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<BinaryColor> {
        if x >= WIDTH || y >= HEIGHT {
            return None;
        }

        let index = y * BYTES_PER_ROW + x / 8;
        let mask = 0x80 >> (x % 8);

        Some(if self.data[index] & mask == 0 {
            BinaryColor::On
        } else {
            BinaryColor::Off
        })
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl DrawTarget for FrameBuffer {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.y >= 0 {
                self.set_pixel(point.x as usize, point.y as usize, color);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_has_correct_size() {
        let buffer = FrameBuffer::new(0xFF);

        assert_eq!(FRAME_BYTES, 48_000);
        assert_eq!(buffer.as_bytes().len(), 48_000);
    }

    #[test]
    fn buffer_initialization() {
        let white = FrameBuffer::new(0xFF);
        let black = FrameBuffer::new(0x00);

        assert!(white.as_bytes().iter().all(|&b| b == 0xFF));
        assert!(black.as_bytes().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn clear_changes_all_bytes() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.clear(0x00);

        assert!(buffer.as_bytes().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn first_pixel_is_msb() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.set_pixel(0, 0, BinaryColor::On);

        assert_eq!(buffer.as_bytes()[0], 0b0111_1111);
    }

    #[test]
    fn last_pixel_in_byte_is_lsb() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.set_pixel(7, 0, BinaryColor::On);

        assert_eq!(buffer.as_bytes()[0], 0b1111_1110);
    }

    #[test]
    fn pixels_are_stored_row_by_row() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.set_pixel(0, 1, BinaryColor::On);

        assert_eq!(buffer.as_bytes()[BYTES_PER_ROW], 0b0111_1111);
    }

    #[test]
    fn set_and_get_pixel() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.set_pixel(10, 20, BinaryColor::On);

        assert_eq!(buffer.get_pixel(10, 20), Some(BinaryColor::On));

        assert_eq!(buffer.get_pixel(11, 20), Some(BinaryColor::Off));
    }

    #[test]
    fn out_of_bounds_pixels_are_ignored() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer.set_pixel(WIDTH, 0, BinaryColor::On);
        buffer.set_pixel(0, HEIGHT, BinaryColor::On);

        assert!(buffer.as_bytes().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn out_of_bounds_get_returns_none() {
        let buffer = FrameBuffer::new(0xFF);

        assert_eq!(buffer.get_pixel(WIDTH, 0), None);
        assert_eq!(buffer.get_pixel(0, HEIGHT), None);
    }

    #[test]
    fn draw_target_writes_pixels() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer
            .draw_iter([
                Pixel(Point::new(0, 0), BinaryColor::On),
                Pixel(Point::new(1, 0), BinaryColor::On),
                Pixel(Point::new(2, 0), BinaryColor::Off),
            ])
            .unwrap();

        assert_eq!(buffer.as_bytes()[0], 0b0011_1111);
    }

    #[test]
    fn draw_target_ignores_negative_coordinates() {
        let mut buffer = FrameBuffer::new(0xFF);

        buffer
            .draw_iter([
                Pixel(Point::new(-1, 0), BinaryColor::On),
                Pixel(Point::new(0, -1), BinaryColor::On),
            ])
            .unwrap();

        assert_eq!(buffer.as_bytes()[0], 0xFF);
    }

    #[test]
    fn dimensions_are_correct() {
        let buffer = FrameBuffer::new(0xFF);

        assert_eq!(buffer.size(), Size::new(800, 480));
    }
}
