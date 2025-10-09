use crate::patterns::mapping::Mapping;
use crate::patterns::texture::Texture;
use crate::primitives::{Coordinates, Transformation};
use crate::{impl_color_provider_for_texture, impl_transform_for};
use core::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CheckeredTexture {
    color_a: Color,
    color_b: Color,
    width: i32,
    height: i32,
    mapping: Arc<dyn Mapping>,
    transformation_inverse: Transformation,
}

impl CheckeredTexture {
    pub fn new(
        color_a: Color,
        color_b: Color,
        width: i32,
        height: i32,
        mapping: Arc<dyn Mapping>,
    ) -> Self {
        return Self {
            color_a,
            color_b,
            width,
            height,
            mapping,
            transformation_inverse: Transformation::IDENTITY,
        };
    }

    pub fn color_at(&self, coordinates: &Coordinates) -> Color {
        let u2 = (coordinates.u * self.width as f64) as i32;
        let v2 = (coordinates.v * self.width as f64) as i32;
        if u2 + v2 % 1 == 0 {
            return self.color_a;
        }
        return self.color_b;
    }
}

impl_transform_for!(CheckeredTexture);

impl_color_provider_for_texture!(CheckeredTexture);

impl Texture for CheckeredTexture {
    fn color_at(&self, point: &Coordinates) -> Color {
        return self.color_at(point);
    }

    fn mapping(&self) -> &dyn Mapping {
        return self.mapping.as_ref();
    }
}

impl Display for CheckeredTexture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return f
            .debug_struct("CheckeredTexture")
            .field("color_a", &self.color_a)
            .field("color_b", &self.color_b)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("mapping", self.mapping.as_ref())
            .finish();
    }
}

impl PartialEq for CheckeredTexture {
    fn eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.color_a == other.color_a
                && self.color_b == other.color_b
                && self.width == other.width
                && self.height == other.height
                && self.mapping.as_ref() == other.mapping.as_ref()
                && self.transformation_inverse == other.transformation_inverse;
    }
}
