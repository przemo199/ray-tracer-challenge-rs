use crate::impl_transform_for;
use crate::patterns::{ColorProvider, UvPattern};
use crate::primitives::{Color, Coordinates, Point, Transformation};
use crate::shapes::{Shape, Transform};
use core::fmt::{Display, Formatter};

/// A face of the unit cube (`-1..=1` on every axis). `Front` is `+z`, `Back` is `-z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeFace {
    Left,
    Right,
    Front,
    Back,
    Up,
    Down,
}

impl CubeFace {
    /// The face a point is on (or nearest to): the axis with the largest magnitude.
    pub fn of(point: &Point) -> Self {
        let largest = point.x.abs().max(point.y.abs()).max(point.z.abs());
        return if largest == point.x {
            Self::Right
        } else if largest == -point.x {
            Self::Left
        } else if largest == point.y {
            Self::Up
        } else if largest == -point.y {
            Self::Down
        } else if largest == point.z {
            Self::Front
        } else {
            Self::Back
        };
    }

    /// Texture coordinates of a point on this face (the book's convention). Each face shows a whole
    /// UV pattern, upright when seen from *inside* the cube, as in a skybox: facing `Front` (`+z`) with
    /// `+y` up, then turning to each face. Seen from outside, faces appear mirrored.
    pub fn coordinates(self, point: &Point) -> Coordinates {
        // maps -1..=1 to 0..=1, wrapping like the other mappings
        let unit = |value: f64| value.rem_euclid(2.0) / 2.0;
        let (u, v) = match self {
            Self::Front => (point.x + 1.0, point.y + 1.0),
            Self::Back => (1.0 - point.x, point.y + 1.0),
            Self::Left => (point.z + 1.0, point.y + 1.0),
            Self::Right => (1.0 - point.z, point.y + 1.0),
            Self::Up => (point.x + 1.0, 1.0 - point.z),
            Self::Down => (point.x + 1.0, point.z + 1.0),
        };
        return Coordinates::new(unit(u), unit(v));
    }
}

/// A different [`UvPattern`] on each face of a cube (e.g. the six images of a skybox).
#[derive(Clone, Debug)]
pub struct CubeMap {
    transformation_inverse: Transformation,
    pub left: Box<dyn UvPattern>,
    pub front: Box<dyn UvPattern>,
    pub right: Box<dyn UvPattern>,
    pub back: Box<dyn UvPattern>,
    pub up: Box<dyn UvPattern>,
    pub down: Box<dyn UvPattern>,
}

impl CubeMap {
    pub fn new(
        left: Box<dyn UvPattern>,
        front: Box<dyn UvPattern>,
        right: Box<dyn UvPattern>,
        back: Box<dyn UvPattern>,
        up: Box<dyn UvPattern>,
        down: Box<dyn UvPattern>,
    ) -> Self {
        return Self {
            transformation_inverse: Transformation::IDENTITY,
            left,
            front,
            right,
            back,
            up,
            down,
        };
    }

    pub fn face(&self, face: CubeFace) -> &dyn UvPattern {
        return match face {
            CubeFace::Left => self.left.as_ref(),
            CubeFace::Front => self.front.as_ref(),
            CubeFace::Right => self.right.as_ref(),
            CubeFace::Back => self.back.as_ref(),
            CubeFace::Up => self.up.as_ref(),
            CubeFace::Down => self.down.as_ref(),
        };
    }
}

// Not derived: a derived `==` on `Box<dyn …>` fields tries to move the boxes (rust-lang/rust#31740).
impl PartialEq for CubeMap {
    fn eq(&self, other: &Self) -> bool {
        const FACES: [CubeFace; 6] = [
            CubeFace::Left,
            CubeFace::Front,
            CubeFace::Right,
            CubeFace::Back,
            CubeFace::Up,
            CubeFace::Down,
        ];
        return self.transformation_inverse == other.transformation_inverse
            && FACES
                .iter()
                .all(|&face| self.face(face) == other.face(face));
    }
}

impl_transform_for!(CubeMap);

impl ColorProvider for CubeMap {
    fn color_at_shape(&self, shape: &dyn Shape, point: &Point) -> Color {
        let object_point = shape.transformation_inverse() * *point;
        let pattern_point = self.transformation_inverse * object_point;
        let face = CubeFace::of(&pattern_point);
        return self.face(face).color_at(&face.coordinates(&pattern_point));
    }
}

impl Display for CubeMap {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter
            .debug_struct("CubeMap")
            .field("left", &self.left)
            .field("front", &self.front)
            .field("right", &self.right)
            .field("back", &self.back)
            .field("up", &self.up)
            .field("down", &self.down)
            .field("transformation", &self.transformation())
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::UvAlignCheck;
    use crate::primitives::transformations;
    use crate::shapes::Cube;
    use rstest::rstest;

    #[rstest]
    #[case(Point::new(-1, 0.5, -0.25), CubeFace::Left)]
    #[case(Point::new(1.1, -0.75, 0.8), CubeFace::Right)]
    #[case(Point::new(0.1, 0.6, 0.9), CubeFace::Front)]
    #[case(Point::new(-0.7, 0, -2), CubeFace::Back)]
    #[case(Point::new(0.5, 1, 0.9), CubeFace::Up)]
    #[case(Point::new(-0.2, -1.3, 1.1), CubeFace::Down)]
    fn identifying_the_face_of_a_cube_from_a_point(#[case] point: Point, #[case] face: CubeFace) {
        assert_eq!(CubeFace::of(&point), face);
    }

    #[rstest]
    #[case(CubeFace::Front, Point::new(-0.5, 0.5, 1), (0.25, 0.75))]
    #[case(CubeFace::Front, Point::new(0.5, -0.5, 1), (0.75, 0.25))]
    #[case(CubeFace::Back, Point::new(0.5, 0.5, -1), (0.25, 0.75))]
    #[case(CubeFace::Back, Point::new(-0.5, -0.5, -1), (0.75, 0.25))]
    #[case(CubeFace::Left, Point::new(-1, 0.5, -0.5), (0.25, 0.75))]
    #[case(CubeFace::Left, Point::new(-1, -0.5, 0.5), (0.75, 0.25))]
    #[case(CubeFace::Right, Point::new(1, 0.5, 0.5), (0.25, 0.75))]
    #[case(CubeFace::Right, Point::new(1, -0.5, -0.5), (0.75, 0.25))]
    #[case(CubeFace::Up, Point::new(-0.5, 1, -0.5), (0.25, 0.75))]
    #[case(CubeFace::Up, Point::new(0.5, 1, 0.5), (0.75, 0.25))]
    #[case(CubeFace::Down, Point::new(-0.5, -1, 0.5), (0.25, 0.75))]
    #[case(CubeFace::Down, Point::new(0.5, -1, -0.5), (0.75, 0.25))]
    fn uv_mapping_of_each_cube_face(
        #[case] face: CubeFace,
        #[case] point: Point,
        #[case] (u, v): (f64, f64),
    ) {
        assert_eq!(face.coordinates(&point), Coordinates::new(u, v));
    }

    const YELLOW: Color = Color {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
    };

    /// Each face: its own main color, the same four corner colors.
    fn align_checks() -> CubeMap {
        let face = |main: Color| -> Box<dyn UvPattern> {
            return Box::new(UvAlignCheck::new(
                main,
                Color::RED,
                YELLOW,
                Color::GREEN,
                Color::WHITE,
            ));
        };
        return CubeMap::new(
            face(Color::new(0.1, 0, 0)),
            face(Color::new(0.2, 0, 0)),
            face(Color::new(0.3, 0, 0)),
            face(Color::new(0.4, 0, 0)),
            face(Color::new(0.5, 0, 0)),
            face(Color::new(0.6, 0, 0)),
        );
    }

    #[rstest]
    #[case(Point::new(-1, 0, 0), Color::new(0.1, 0, 0))]
    #[case(Point::new(0, 0, 1), Color::new(0.2, 0, 0))]
    #[case(Point::new(1, 0, 0), Color::new(0.3, 0, 0))]
    #[case(Point::new(0, 0, -1), Color::new(0.4, 0, 0))]
    #[case(Point::new(0, 1, 0), Color::new(0.5, 0, 0))]
    #[case(Point::new(0, -1, 0), Color::new(0.6, 0, 0))]
    // front face: u grows with x, v with y
    #[case(Point::new(-0.9, 0.9, 1), Color::RED)]
    #[case(Point::new(0.9, 0.9, 1), YELLOW)]
    #[case(Point::new(-0.9, -0.9, 1), Color::GREEN)]
    #[case(Point::new(0.9, -0.9, 1), Color::WHITE)]
    // back face: u grows towards -x
    #[case(Point::new(0.9, 0.9, -1), Color::RED)]
    #[case(Point::new(-0.9, -0.9, -1), Color::WHITE)]
    // top face: v grows towards -z
    #[case(Point::new(-0.9, 1, -0.9), Color::RED)]
    // bottom face: v grows towards +z
    #[case(Point::new(-0.9, -1, 0.9), Color::RED)]
    fn each_face_shows_its_own_pattern(#[case] point: Point, #[case] expected: Color) {
        assert_eq!(
            align_checks().color_at_shape(&Cube::default(), &point),
            expected
        );
    }

    #[test]
    fn cube_map_transformation_applies() {
        let mut cube_map = align_checks();
        // a quarter turn about y brings the right face (+x) to the front (+z)
        cube_map.set_transformation(transformations::rotation_y(-core::f64::consts::FRAC_PI_2));
        let color = cube_map.color_at_shape(&Cube::default(), &Point::new(0, 0, 1));
        assert_eq!(color, Color::new(0.3, 0, 0));
    }

    #[test]
    fn cube_maps_compare_face_by_face() {
        let mut other = align_checks();
        assert!(other == align_checks());
        other.up = Box::new(UvAlignCheck::new(YELLOW, YELLOW, YELLOW, YELLOW, YELLOW));
        assert!(other != align_checks());
    }
}
