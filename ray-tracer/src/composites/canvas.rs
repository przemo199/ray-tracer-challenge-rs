use crate::composites::World;
use crate::primitives::Color;
use core::ops::Deref;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;
use core::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub(crate) pixels: Vec<Color>,
}

impl Canvas {
    pub const DEFAULT_COLOR: Color = World::DEFAULT_COLOR;

    pub const MIN_COLOR_VALUE: f64 = Color::MIN_COLOR_VALUE;

    pub const MAX_COLOR_VALUE: f64 = 255.0;

    /// Creates new instance of struct Canvas
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        let pixels = vec![Self::DEFAULT_COLOR; pixel_count];
        return Self {
            width,
            height,
            pixels,
        };
    }

    pub const fn xy_to_index(&self, x: u32, y: u32) -> usize {
        return Self::coords_to_index(self.width, (x, y));
    }

    pub const fn coords_to_index(canvas_width: u32, (x, y): (u32, u32)) -> usize {
        return ((y * canvas_width) + x) as usize;
    }

    pub const fn index_to_xy(&self, index: usize) -> (u32, u32) {
        return Self::index_to_coords(self.width, index);
    }

    pub const fn index_to_coords(canvas_width: u32, index: usize) -> (u32, u32) {
        return (index as u32 % canvas_width, index as u32 / canvas_width);
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> &Color {
        return &self.pixels[self.xy_to_index(x, y)];
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = self.xy_to_index(x, y);
        self.pixels[index] = color;
    }

    fn get_header(&self) -> Vec<String> {
        let identifier = "P3".to_owned();
        let color_range = (Self::MAX_COLOR_VALUE as i64).to_string();
        let image_size = [self.width.to_string(), self.height.to_string()].join(" ");
        return vec![identifier, image_size, color_range];
    }

    fn to_ppm(&self) -> String {
        let ppm_image = self
            .pixels
            .chunks(self.width as usize)
            .into_iter()
            .map(|line| {
                return line
                    .iter()
                    .map(Color::normalized)
                    .flat_map(|color| color.channels().into_iter())
                    .map(|channel: f64| {
                        ((channel * Self::MAX_COLOR_VALUE).round() as i64).to_string()
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
            });
        let mut content = self.get_header();
        content.extend(ppm_image);
        return content.join("\n");
    }

    fn prepare_file<P: AsRef<Path>>(file_name: &P) -> io::Result<()> {
        let path = Path::new(file_name.as_ref());
        let prefix = path
            .parent()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        return std::fs::create_dir_all(prefix);
    }

    pub fn to_ppm_file<P: AsRef<Path>>(&self, file_name: P) -> io::Result<()> {
        Self::prepare_file(&file_name)?;
        let content = self.to_ppm();
        let mut file = File::create(file_name.as_ref())?;
        return file.write_all(content.as_bytes());
    }

    pub fn to_png_file<P: AsRef<Path>>(&self, file_name: P) -> Result<(), Box<dyn Error>> {
        Self::prepare_file(&file_name)?;

        let buffer: Vec<u8> = self
            .pixels
            .iter()
            .map(Color::normalized)
            .flat_map(|color| color.channels().into_iter())
            .map(|channel| (channel * Self::MAX_COLOR_VALUE).round() as u8)
            .collect();

        let buf_file_writer = BufWriter::new(File::create(file_name.as_ref())?);
        let encoder = PngEncoder::new_with_quality(
            buf_file_writer,
            CompressionType::Best,
            FilterType::NoFilter,
        );
        return Ok(encoder.write_image(
            buffer.as_slice(),
            self.width,
            self.height,
            image::ExtendedColorType::Rgb8,
        )?);
    }
}

impl Deref for Canvas {
    type Target = Vec<Color>;

    fn deref(&self) -> &Self::Target {
        return &self.pixels;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_canvas() {
        let canvas = Canvas::new(10, 20);
        assert_eq!(canvas.width, 10);
        assert_eq!(canvas.height, 20);
        assert_eq!(canvas.pixels.len(), 200);
        let black = Color::BLACK;
        for pixel in &canvas.pixels {
            assert_eq!(pixel, &black);
        }
    }

    #[test]
    fn set_pixel() {
        let mut canvas = Canvas::new(10, 20);
        canvas.set_pixel(2, 3, Color::RED);
        assert_eq!(canvas.get_pixel(2, 3), &Color::new(1, 0, 0));
    }

    #[test]
    fn get_header() {
        let canvas = Canvas::new(5, 3);
        let header = canvas.get_header();
        assert_eq!(header[0], "P3");
        assert_eq!(header[1], "5 3");
        assert_eq!(header[2], "255");
    }

    #[test]
    fn to_ppm() {
        let mut canvas = Canvas::new(5, 3);
        let color_1 = Color::new(1.5, 0, 0);
        let color_2 = Color::new(0, 0.5, 0);
        let color_3 = Color::new(-0.5, 0, 1);
        canvas.set_pixel(0, 0, color_1);
        canvas.set_pixel(2, 1, color_2);
        canvas.set_pixel(4, 2, color_3);
        let ppm: Vec<String> = canvas.to_ppm().lines().map(str::to_owned).collect();
        assert_eq!(ppm[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(ppm[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(ppm[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }
}
