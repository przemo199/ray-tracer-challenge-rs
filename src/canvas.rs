use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use image::codecs::png::*;
use image::ImageEncoder;

use crate::primitives::Color;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Canvas {
    /// Creates new instance of struct Canvas
    pub fn new(width: u32, height: u32) -> Canvas {
        let pixel_count = (width * height) as usize;
        let pixels = vec![Color::BLACK; pixel_count];
        return Canvas {
            width,
            height,
            pixels,
        };
    }

    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        return (x + (y * self.width)) as usize;
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> &Color {
        return &self.pixels[self.coords_to_index(x, y)];
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = self.coords_to_index(x, y);
        self.pixels[index] = color;
    }

    fn get_header(&self) -> Vec<String> {
        let identifier = "P3".to_string();
        let color_range = "255".to_string();
        let image_size = [self.width.to_string(), self.height.to_string()].join(" ");
        return vec![identifier, image_size, color_range];
    }

    fn to_ppm(&self) -> String {
        let mut content: Vec<String> = self.get_header();
        let max_color_value: f64 = 1.0;

        for line in self.pixels.chunks(self.width as usize) {
            let mut line_content: Vec<String> = Vec::new();
            for pixel in line {
                for mut color_value in pixel.get_channels() {
                    if color_value < 0.0 {
                        color_value = 0.0;
                    }

                    if color_value > 1.0 {
                        color_value = 1.0;
                    }

                    let scaled_color_value = ((color_value / max_color_value) * 255.0).round() as i32;
                    line_content.push(scaled_color_value.to_string());
                }
            }
            content.push(line_content.join(" "));
        }

        return content.join("\n");
    }

    fn prepare_file<P: AsRef<Path>>(&self, file_name: &P) {
        let path = Path::new(file_name.as_ref());
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
    }

    pub fn to_ppm_file<P: AsRef<Path>>(&self, file_name: P) {
        self.prepare_file(&file_name);
        let content = self.to_ppm();
        let mut file = File::create(file_name.as_ref()).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn to_png_file<P: AsRef<Path>>(&self, file_name: P) {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.pixels.len() * 3);
        self.prepare_file(&file_name);

        for pixel in self.pixels.iter() {
            for color in pixel.get_channels() {
                buffer.push((color * 255.0).round() as u8);
            }
        }

        let buf_file_writer = BufWriter::new(File::create(file_name.as_ref()).unwrap());
        let encoder = PngEncoder::new_with_quality(buf_file_writer, CompressionType::Best, FilterType::NoFilter);
        encoder.write_image(buffer.as_slice(), self.width, self.height, image::ColorType::Rgb8).unwrap();
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
        for pixel in canvas.pixels.iter() {
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
        let ppm: Vec<String> = canvas.to_ppm().lines().map(|x| x.to_string()).collect();
        assert_eq!(ppm[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(ppm[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(ppm[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }
}
