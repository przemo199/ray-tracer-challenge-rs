use crate::patterns::mapping::Mapping;
use crate::patterns::texture::Texture;
use crate::primitives::{Coordinates, Transformation};
use crate::shapes::Transform;
use crate::{impl_color_provider_for_texture, impl_transform_for};
use core::fmt::{Display, Formatter};
use image::ImageBuffer;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CustomTexture {
    pub texture: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub mapping: Box<dyn Mapping>,
    transformation_inverse: Transformation,
}

impl CustomTexture {
    pub fn from_buffer(
        image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        mapping: Box<dyn Mapping>,
    ) -> Self {
        return CustomTexture {
            texture: image,
            mapping,
            transformation_inverse: Transformation::IDENTITY,
        };
    }

    pub fn from_path<P: AsRef<Path>>(path: P, mapping: Box<dyn Mapping>) -> Self {
        let image = image::open(path).expect("Failed to open image").to_rgba8();
        return CustomTexture {
            texture: image,
            mapping,
            transformation_inverse: Transformation::IDENTITY,
        };
    }

    pub fn color_at(&self, coordinates: &Coordinates) -> Color {
        let x = (coordinates.u * self.texture.width() as f64) as u32;
        let y = (coordinates.v * self.texture.height() as f64) as u32;
        return self.texture.get_pixel(x, y).0.into();
    }
}

impl_transform_for!(CustomTexture);

impl_color_provider_for_texture!(CustomTexture);

impl Texture for CustomTexture {
    fn color_at(&self, point: &Coordinates) -> Color {
        return self.color_at(point);
    }

    fn mapping(&self) -> &dyn Mapping {
        return self.mapping.as_ref();
    }
}

impl Display for CustomTexture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return f
            .debug_struct("CustomTexture")
            .field(
                "texture",
                &(self.texture.width().to_string() + "x" + &self.texture.height().to_string()),
            )
            .field("mapping", self.mapping.as_ref())
            .finish();
    }
}

impl PartialEq for CustomTexture {
    fn eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.texture == other.texture
                && self.mapping.as_ref() == other.mapping.as_ref()
                && self.transformation_inverse == other.transformation_inverse;
    }
}
