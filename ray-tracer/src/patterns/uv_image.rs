use crate::patterns::UvPattern;
use crate::primitives::{Color, Coordinates};
use core::fmt::{Debug, Formatter};
use image::{ImageResult, RgbImage};
use std::path::Path;

/// An image used as a texture: `u` runs left to right, `v` from the bottom row (0) to the top (1).
/// Pixels are looked up nearest-neighbor.
#[derive(Clone, PartialEq)]
pub struct UvImage {
    image: RgbImage,
}

impl UvImage {
    /// # Panics
    /// If the image is empty (0 pixels wide or high).
    pub fn new(image: RgbImage) -> Self {
        assert!(
            image.width() > 0 && image.height() > 0,
            "texture image must not be empty"
        );
        return Self { image };
    }

    /// Loads a PNG or JPEG file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> ImageResult<Self> {
        return Ok(Self::new(image::open(path)?.to_rgb8()));
    }

    pub fn width(&self) -> u32 {
        return self.image.width();
    }

    pub fn height(&self) -> u32 {
        return self.image.height();
    }
}

impl UvPattern for UvImage {
    fn color_at(&self, coordinates: &Coordinates) -> Color {
        let last_x = f64::from(self.image.width() - 1);
        let last_y = f64::from(self.image.height() - 1);
        // clamp so u, v = 1.0 (and stray values from a mapping) stay inside the image; image row 0
        // is the top, while v = 0 is the bottom
        let x = (coordinates.u.clamp(0.0, 1.0) * last_x).round() as u32;
        let y = ((1.0 - coordinates.v.clamp(0.0, 1.0)) * last_y).round() as u32;
        let [red, green, blue] = self.image.get_pixel(x, y).0;
        let channel = |value: u8| f64::from(value) / 255.0;
        return Color::new(channel(red), channel(green), channel(blue));
    }
}

// The derived `Debug` would print every pixel.
impl Debug for UvImage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter
            .debug_struct("UvImage")
            .field("width", &self.image.width())
            .field("height", &self.image.height())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;
    use rstest::rstest;

    /// 2×2 image: red and green on the top row, blue and white on the bottom row.
    fn corners() -> UvImage {
        let mut image = RgbImage::new(2, 2);
        image.put_pixel(0, 0, Rgb([255, 0, 0]));
        image.put_pixel(1, 0, Rgb([0, 255, 0]));
        image.put_pixel(0, 1, Rgb([0, 0, 255]));
        image.put_pixel(1, 1, Rgb([255, 255, 255]));
        return UvImage::new(image);
    }

    #[rstest]
    #[case(0.0, 1.0, Color::RED)]
    #[case(1.0, 1.0, Color::GREEN)]
    #[case(0.0, 0.0, Color::BLUE)]
    #[case(1.0, 0.0, Color::WHITE)]
    #[case(0.4, 0.6, Color::RED)]
    #[case(0.6, 0.4, Color::WHITE)]
    fn v_runs_from_the_bottom_row_to_the_top(
        #[case] u: f64,
        #[case] v: f64,
        #[case] expected: Color,
    ) {
        assert_eq!(corners().color_at(&Coordinates::new(u, v)), expected);
    }

    #[test]
    fn channels_are_scaled_to_0_1() {
        let image = UvImage::new(RgbImage::from_pixel(1, 1, Rgb([255, 51, 0])));
        let color = image.color_at(&Coordinates::new(0.5, 0.5));
        assert_eq!(color, Color::new(1, 0.2, 0));
    }

    #[test]
    fn coordinates_outside_0_1_are_clamped() {
        let image = corners();
        assert_eq!(image.color_at(&Coordinates::new(1.5, 2)), Color::GREEN);
        assert_eq!(image.color_at(&Coordinates::new(-0.5, -1)), Color::BLUE);
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(UvImage::from_path("does-not-exist.png").is_err());
    }
}
