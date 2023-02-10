use std::fs::File;
use std::io::Write;
use std::path::Path;
use image::ImageFormat;
use crate::color::Color;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        let pixel_count = (width * height) as usize;
        let pixels = vec![Color::black(); pixel_count];
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
                for mut color_value in pixel.get_colors() {
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

    pub fn to_ppm_file<P: AsRef<Path>>(&self, file_name: P) {
        let content = self.to_ppm();
        let mut file = File::create(file_name.as_ref()).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    pub fn to_png_file<P: AsRef<Path>>(&self, file_name: P) {
        let mut buffer: Vec<u8> = Vec::new();

        for pixel in self.pixels.iter() {
            for color in pixel.get_colors() {
                buffer.push((color * 255.0).round() as u8);
            }
        }

        image::save_buffer_with_format(
            file_name.as_ref(),
            buffer.as_slice(),
            self.width,
            self.height,
            image::ColorType::Rgb8,
            ImageFormat::Png,
        ).unwrap();
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
        let black = Color::black();
        for pixel in canvas.pixels.iter() {
            assert_eq!(pixel, &black);
        }
    }

    #[test]
    fn set_pixel() {
        let mut canvas = Canvas::new(10, 20);
        canvas.set_pixel(2, 3, Color::red());
        assert_eq!(canvas.get_pixel(2, 3), &Color::new(1.0, 0.0, 0.0));
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
        let color1 = Color::new(1.5, 0.0, 0.0);
        let color2 = Color::new(0.0, 0.5, 0.0);
        let color3 = Color::new(-0.5, 0.0, 1.0);
        canvas.set_pixel(0, 0, color1);
        canvas.set_pixel(2, 1, color2);
        canvas.set_pixel(4, 2, color3);
        let ppm: Vec<String> = canvas.to_ppm().lines().map(|x| x.to_string()).collect();
        assert_eq!(ppm[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(ppm[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(ppm[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }
}
