use crate::primitives::Color;
use core::fmt::Write as _;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder, ImageResult};
use std::fs::{self, File};
use std::io;
use std::path::Path;

/// A grid of pixels, stored row by row starting at the top-left corner.
#[derive(Clone, Debug, PartialEq)]
pub struct Canvas {
    // Private: `pixels` holds exactly `width * height` colors, so the size can't change alone.
    width: u32,
    height: u32,
    pub(crate) pixels: Vec<Color>,
}

impl Canvas {
    pub const DEFAULT_COLOR: Color = Color::BLACK;

    pub const MIN_COLOR_VALUE: f64 = Color::MIN_COLOR_VALUE;

    /// Largest 8-bit channel value, used by the PPM and PNG output.
    pub const MAX_COLOR_VALUE: f64 = 255.0;

    /// PPM lines must stay within 70 characters: 5 pixels × 3 channels × "255 " is 59.
    const PPM_PIXELS_PER_LINE: usize = 5;

    /// Creates a canvas with every pixel set to [`Canvas::DEFAULT_COLOR`].
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = width as usize * height as usize;
        return Self {
            width,
            height,
            pixels: vec![Self::DEFAULT_COLOR; pixel_count],
        };
    }

    #[inline]
    pub const fn width(&self) -> u32 {
        return self.width;
    }

    #[inline]
    pub const fn height(&self) -> u32 {
        return self.height;
    }

    #[inline]
    pub const fn xy_to_index(&self, x: u32, y: u32) -> usize {
        return Self::coords_to_index(self.width, (x, y));
    }

    #[inline]
    pub const fn coords_to_index(canvas_width: u32, (x, y): (u32, u32)) -> usize {
        return x as usize + y as usize * canvas_width as usize;
    }

    #[inline]
    pub const fn index_to_xy(&self, index: usize) -> (u32, u32) {
        return Self::index_to_coords(self.width, index);
    }

    #[inline]
    pub const fn index_to_coords(canvas_width: u32, index: usize) -> (u32, u32) {
        let width = canvas_width as usize;
        return ((index % width) as u32, (index / width) as u32);
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> &Color {
        return &self.pixels[self.xy_to_index(x, y)];
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = self.xy_to_index(x, y);
        self.pixels[index] = color;
    }

    /// All pixels, row by row.
    pub fn pixels(&self) -> &[Color] {
        return &self.pixels;
    }

    /// The pixels as 8-bit RGB triples (row by row), with channels clamped to `0..=1` and rounded.
    /// This is exactly what [`Canvas::to_png_file`] writes.
    pub fn to_rgb8(&self) -> Vec<u8> {
        return self
            .pixels
            .iter()
            .flat_map(|color| color.clamped().channels())
            .map(|channel| (channel * Self::MAX_COLOR_VALUE).round() as u8)
            .collect();
    }

    /// The canvas as a plain-text PPM (P3) image.
    fn to_ppm(&self) -> String {
        let max = Self::MAX_COLOR_VALUE as u8;
        let mut ppm = format!("P3\n{} {}\n{max}\n", self.width, self.height);
        for line in self.to_rgb8().chunks(Self::PPM_PIXELS_PER_LINE * 3) {
            let numbers: Vec<String> = line.iter().map(|value| format!("{value:>3}")).collect();
            // writing to a String cannot fail
            let _ = writeln!(ppm, "{}", numbers.join(" "));
        }
        return ppm;
    }

    /// Creates the parent directories of `path` if needed.
    fn create_parent_dir(path: &Path) -> io::Result<()> {
        return match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => fs::create_dir_all(parent),
            _ => Ok(()),
        };
    }

    /// Writes the canvas as a plain-text PPM file, creating parent directories if needed.
    pub fn to_ppm_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        Self::create_parent_dir(path)?;
        return fs::write(path, self.to_ppm());
    }

    /// Writes the canvas as an 8-bit RGB PNG file, creating parent directories if needed.
    pub fn to_png_file<P: AsRef<Path>>(&self, path: P) -> ImageResult<()> {
        let path = path.as_ref();
        Self::create_parent_dir(path)?;
        let writer = File::create(path)?;
        let encoder =
            PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::NoFilter);
        return encoder.write_image(
            &self.to_rgb8(),
            self.width,
            self.height,
            ExtendedColorType::Rgb8,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_canvas() {
        let canvas = Canvas::new(10, 20);
        assert_eq!(canvas.width(), 10);
        assert_eq!(canvas.height(), 20);
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
        assert_eq!(canvas.get_pixel(2, 3), &Color::RED);
    }

    #[test]
    fn ppm_header() {
        let ppm = Canvas::new(5, 3).to_ppm();
        let header: Vec<&str> = ppm.lines().take(3).collect();
        assert_eq!(header, ["P3", "5 3", "255"]);
    }

    #[test]
    fn rgb8_clamps_and_rounds_channels() {
        let mut canvas = Canvas::new(3, 1);
        canvas.set_pixel(0, 0, Color::new(1.5, 0.5, -0.5));
        canvas.set_pixel(1, 0, Color::new(0.2, 0.4, 0.6));
        assert_eq!(canvas.to_rgb8(), [255, 128, 0, 51, 102, 153, 0, 0, 0]);
    }

    #[test]
    fn index_math_does_not_overflow_u32() {
        let width = 100_000;
        let index = Canvas::coords_to_index(width, (1, 50_000));
        assert_eq!(index, 5_000_000_001);
        assert_eq!(Canvas::index_to_coords(width, index), (1, 50_000));
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
        assert_eq!(
            ppm[3],
            "255   0   0   0   0   0   0   0   0   0   0   0   0   0   0"
        );
        assert_eq!(
            ppm[4],
            "  0   0   0   0   0   0   0 128   0   0   0   0   0   0   0"
        );
        assert_eq!(
            ppm[5],
            "  0   0   0   0   0   0   0   0   0   0   0   0   0   0 255"
        );
    }

    #[test]
    fn splitting_long_lines_in_ppm() {
        let mut canvas = Canvas::new(10, 2);
        canvas.pixels.fill(Color::new(1, 0.8, 0.6));
        let ppm = canvas.to_ppm();
        assert!(ppm.lines().all(|line| line.len() <= 70), "{ppm}");
        let numbers: Vec<&str> = ppm
            .lines()
            .skip(3)
            .flat_map(str::split_whitespace)
            .collect();
        let (pixels, remainder) = numbers.as_chunks::<3>();
        assert_eq!((pixels.len(), remainder.len()), (10 * 2, 0));
        assert!(pixels.iter().all(|pixel| *pixel == ["255", "204", "153"]));
    }

    #[test]
    fn ppm_ends_with_newline() {
        assert!(Canvas::new(5, 3).to_ppm().ends_with('\n'));
    }
}
