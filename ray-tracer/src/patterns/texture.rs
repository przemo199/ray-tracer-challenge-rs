use crate::patterns::ColorProvider;
use crate::patterns::mapping::Mapping;
use crate::primitives::{Color, Coordinates};
use crate::shapes::Transform;

pub trait Texture: Transform + ColorProvider {
    fn color_at(&self, point: &Coordinates) -> Color;

    fn mapping(&self) -> &dyn Mapping;
}

#[macro_export]
macro_rules! impl_color_provider_for_texture {
    ($type:ty) => {
        use $crate::patterns::color_provider::ColorProvider;
        use $crate::primitives::{Color, Point};
        use $crate::shapes::Shape;

        impl ColorProvider for $type {
            fn color_at_shape(&self, shape: &dyn Shape, point: &Point) -> Color {
                let object_point = shape.transformation_inverse() * *point;
                let coordinates = self.mapping().map(&object_point);
                return self.color_at(&coordinates);
            }
        }
    };
}
