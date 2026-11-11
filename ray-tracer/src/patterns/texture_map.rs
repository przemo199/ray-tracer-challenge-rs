use crate::impl_transform_for;
use crate::patterns::{ColorProvider, Mapping, UvPattern};
use crate::primitives::{Color, Point, Transformation};
use crate::shapes::{Shape, Transform};
use core::fmt::{Display, Formatter};

/// Paints a [`UvPattern`] onto shapes: a point is converted to pattern space (shape and texture
/// transformations), mapped to `(u, v)` by the [`Mapping`], and colored by the UV pattern.
#[derive(Clone, Debug)]
pub struct TextureMap {
    transformation_inverse: Transformation,
    pub uv_pattern: Box<dyn UvPattern>,
    pub mapping: Box<dyn Mapping>,
}

// Not derived: a derived `==` on `Box<dyn …>` fields tries to move the boxes (rust-lang/rust#31740).
impl PartialEq for TextureMap {
    fn eq(&self, other: &Self) -> bool {
        return self.transformation_inverse == other.transformation_inverse
            && self.uv_pattern.as_ref() == other.uv_pattern.as_ref()
            && self.mapping.as_ref() == other.mapping.as_ref();
    }
}

impl TextureMap {
    pub fn new(uv_pattern: Box<dyn UvPattern>, mapping: Box<dyn Mapping>) -> Self {
        return Self {
            transformation_inverse: Transformation::IDENTITY,
            uv_pattern,
            mapping,
        };
    }
}

impl_transform_for!(TextureMap);

impl ColorProvider for TextureMap {
    fn color_at_shape(&self, shape: &dyn Shape, point: &Point) -> Color {
        let object_point = shape.transformation_inverse() * *point;
        let pattern_point = self.transformation_inverse * object_point;
        return self.uv_pattern.color_at(&self.mapping.map(&pattern_point));
    }
}

impl Display for TextureMap {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter
            .debug_struct("TextureMap")
            .field("uv_pattern", &self.uv_pattern)
            .field("mapping", &self.mapping)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{PlanarMapping, SphericalMapping, UvCheckers};
    use crate::primitives::transformations;
    use crate::shapes::{Plane, Sphere};
    use rstest::rstest;

    fn checkers(width: u32, height: u32) -> Box<dyn UvPattern> {
        return Box::new(UvCheckers::new(width, height, Color::BLACK, Color::WHITE));
    }

    #[rstest]
    #[case(Point::new(0.4315, 0.4670, 0.7719), Color::WHITE)]
    #[case(Point::new(-0.9654, 0.2552, -0.0534), Color::BLACK)]
    #[case(Point::new(0.1039, 0.7090, 0.6975), Color::WHITE)]
    #[case(Point::new(-0.4986, -0.7856, -0.3663), Color::BLACK)]
    #[case(Point::new(-0.0317, -0.9395, 0.3411), Color::BLACK)]
    #[case(Point::new(0.4809, -0.7721, 0.4154), Color::BLACK)]
    #[case(Point::new(0.0285, -0.9612, -0.2745), Color::BLACK)]
    #[case(Point::new(-0.5734, -0.2162, -0.7903), Color::WHITE)]
    #[case(Point::new(0.7688, -0.1470, 0.6223), Color::BLACK)]
    #[case(Point::new(-0.7652, 0.2175, 0.6060), Color::BLACK)]
    fn checkers_on_a_sphere_with_spherical_mapping(#[case] point: Point, #[case] expected: Color) {
        let texture = TextureMap::new(checkers(16, 8), Box::new(SphericalMapping));
        assert_eq!(texture.color_at_shape(&Sphere::default(), &point), expected);
    }

    #[test]
    fn shape_transformation_applies() {
        let texture = TextureMap::new(checkers(2, 2), Box::new(PlanarMapping));
        let mut plane = Plane::default();
        plane.set_transformation(transformations::translation(0.5, 0, 0));
        // world x = 0.75 is object x = 0.25: the first square, not the second
        let color = texture.color_at_shape(&plane, &Point::new(0.75, 0, 0));
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn texture_transformation_applies() {
        let mut texture = TextureMap::new(checkers(2, 2), Box::new(PlanarMapping));
        let point = Point::new(0.75, 0, 0);
        assert_eq!(
            texture.color_at_shape(&Plane::default(), &point),
            Color::WHITE
        );
        // twice as large squares: x = 0.75 becomes u = 0.375, the first square
        texture.set_transformation(transformations::scaling(2, 2, 2));
        assert_eq!(
            texture.color_at_shape(&Plane::default(), &point),
            Color::BLACK
        );
    }

    #[test]
    fn texture_maps_clone_and_compare_through_trait_objects() {
        let texture: Box<dyn ColorProvider> =
            Box::new(TextureMap::new(checkers(4, 2), Box::new(SphericalMapping)));
        let copy = texture.clone();
        assert!(copy == texture);
        let other: Box<dyn ColorProvider> =
            Box::new(TextureMap::new(checkers(4, 2), Box::new(PlanarMapping)));
        assert!(other != texture);
    }
}
