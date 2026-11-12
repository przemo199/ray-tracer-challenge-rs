use crate::consts::PI;
use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Coordinates, Point, Vector};
use core::fmt::Debug;

/// Maps a point on a shape (in pattern space) to `(u, v)` texture coordinates in `0..=1`, which a
/// [`UvPattern`](crate::patterns::UvPattern) turns into a color. Combined in a
/// [`TextureMap`](crate::patterns::TextureMap).
pub trait Mapping: Debug + Send + Sync + DynPartialEq + CloneMapping {
    fn map(&self, point: &Point) -> Coordinates;
}

/// Makes `Box<dyn Mapping>` cloneable: `Clone` itself can't be called through `dyn`.
/// Implemented automatically for every `Clone` mapping.
pub trait CloneMapping {
    fn clone_box(&self) -> Box<dyn Mapping>;
}

impl<T: Mapping + Clone + 'static> CloneMapping for T {
    fn clone_box(&self) -> Box<dyn Mapping> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<dyn Mapping> {
    fn clone(&self) -> Self {
        return self.clone_box();
    }
}

impl PartialEq for dyn Mapping {
    fn eq(&self, other: &Self) -> bool {
        return self.dyn_eq(DynPartialEq::as_any(other));
    }
}

impl PartialEq<&Self> for Box<dyn Mapping> {
    fn eq(&self, other: &&Self) -> bool {
        return self == *other;
    }
}

impl PartialEq<&Self> for &dyn Mapping {
    fn eq(&self, other: &&Self) -> bool {
        return self == *other;
    }
}

/// Tiles the texture across the xz-plane, repeating every unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PlanarMapping;

impl Mapping for PlanarMapping {
    fn map(&self, point: &Point) -> Coordinates {
        // `rem_euclid`, not `fract`: negative coordinates must wrap into 0..1 too
        return Coordinates::new(point.x.rem_euclid(1.0), point.z.rem_euclid(1.0));
    }
}

/// Wraps the texture around a unit sphere like latitude/longitude: `u` goes around the y-axis,
/// `v` from the south pole (0) to the north pole (1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SphericalMapping;

impl Mapping for SphericalMapping {
    fn map(&self, point: &Point) -> Coordinates {
        let radius = Vector::from(*point).magnitude();
        // polar angle from the north pole, in 0..=π
        let phi = (point.y / radius).acos();
        let v = 1.0 - (phi / PI);
        return Coordinates::new(around_y_axis(point), v);
    }
}

/// Wraps the texture around a unit cylinder like a label on a can: `u` goes around the y-axis, `v`
/// up the side, repeating every unit of height (`v = 0` at `y = 0`, `y = 1`, …).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CylindricalMapping;

impl Mapping for CylindricalMapping {
    fn map(&self, point: &Point) -> Coordinates {
        return Coordinates::new(around_y_axis(point), point.y.rem_euclid(1.0));
    }
}

/// `u` for mappings that wrap around the y-axis: 0 at `-z`, increasing counterclockwise when
/// viewed from above (0.25 at `+x`, 0.5 at `+z`, 0.75 at `-x`).
fn around_y_axis(point: &Point) -> f64 {
    // azimuthal angle, in -π..=π
    let theta = point.x.atan2(point.z);
    let raw_u = theta / (2.0 * PI);
    return 1.0 - (raw_u + 0.5);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::CoarseEq;
    use rstest::rstest;

    #[rstest]
    #[case(Point::new(0.25, 0, 0.5), (0.25, 0.5))]
    #[case(Point::new(0.25, 0, -0.25), (0.25, 0.75))]
    #[case(Point::new(0.25, 0.5, -0.25), (0.25, 0.75))]
    #[case(Point::new(1.25, 0, 0.5), (0.25, 0.5))]
    #[case(Point::new(0.25, 0, -1.75), (0.25, 0.25))]
    #[case(Point::new(1, 0, -1), (0.0, 0.0))]
    #[case(Point::new(0, 0, 0), (0.0, 0.0))]
    fn planar_mapping_on_a_3d_point(#[case] point: Point, #[case] (u, v): (f64, f64)) {
        assert_eq!(PlanarMapping.map(&point), Coordinates::new(u, v));
    }

    #[rstest]
    #[case(Point::new(0, 0, -1), (0.0, 0.5))]
    #[case(Point::new(1, 0, 0), (0.25, 0.5))]
    #[case(Point::new(0, 0, 1), (0.5, 0.5))]
    #[case(Point::new(-1, 0, 0), (0.75, 0.5))]
    #[case(Point::new(0, 1, 0), (0.5, 1.0))]
    #[case(Point::new(0, -1, 0), (0.5, 0.0))]
    #[case(Point::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0), (0.25, 0.75))]
    fn spherical_mapping_on_a_3d_point(#[case] point: Point, #[case] (u, v): (f64, f64)) {
        let coordinates = SphericalMapping.map(&point);
        assert!(coordinates.u.coarse_eq(&u), "{coordinates:?}");
        assert!(coordinates.v.coarse_eq(&v), "{coordinates:?}");
    }

    const HALF_SQRT_2: f64 = core::f64::consts::FRAC_1_SQRT_2;

    #[rstest]
    #[case(Point::new(0, 0, -1), (0.0, 0.0))]
    #[case(Point::new(0, 0.5, -1), (0.0, 0.5))]
    #[case(Point::new(0, 1, -1), (0.0, 0.0))]
    #[case(Point::new(HALF_SQRT_2, 0.5, -HALF_SQRT_2), (0.125, 0.5))]
    #[case(Point::new(1, 0.5, 0), (0.25, 0.5))]
    #[case(Point::new(HALF_SQRT_2, 0.5, HALF_SQRT_2), (0.375, 0.5))]
    #[case(Point::new(0, -0.25, 1), (0.5, 0.75))]
    #[case(Point::new(-HALF_SQRT_2, 0.5, HALF_SQRT_2), (0.625, 0.5))]
    #[case(Point::new(-1, 1.25, 0), (0.75, 0.25))]
    #[case(Point::new(-HALF_SQRT_2, 0.5, -HALF_SQRT_2), (0.875, 0.5))]
    fn cylindrical_mapping_on_a_3d_point(#[case] point: Point, #[case] (u, v): (f64, f64)) {
        let coordinates = CylindricalMapping.map(&point);
        assert!(coordinates.u.coarse_eq(&u), "{coordinates:?}");
        assert!(coordinates.v.coarse_eq(&v), "{coordinates:?}");
    }

    #[test]
    fn cylindrical_and_spherical_mappings_agree_on_u() {
        for point in [Point::new(0.3, 0.2, -0.9), Point::new(-0.8, -0.4, 0.4)] {
            assert_eq!(
                CylindricalMapping.map(&point).u,
                SphericalMapping.map(&point).u
            );
        }
    }
}
