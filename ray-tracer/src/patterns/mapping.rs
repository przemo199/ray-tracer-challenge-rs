use crate::consts::PI;
use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Coordinates, Point, Vector};
use core::fmt::Debug;

pub trait Mapping: Debug + Send + Sync + DynPartialEq {
    fn map(&self, point: &Point) -> Coordinates;
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct PlanarMapping;

impl Mapping for PlanarMapping {
    fn map(&self, point: &Point) -> Coordinates {
        return Coordinates::new(point.x % 1.0, point.z % 1.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SphericalMapping;

impl Mapping for SphericalMapping {
    fn map(&self, point: &Point) -> Coordinates {
        let theta = point.x.atan2(point.z);
        let radius = Into::<Vector>::into(*point).magnitude();
        let phi = (point.y / radius).acos();
        let raw_u = theta / (2.0 * PI);
        let u = 1.0 - (raw_u + 0.5);
        let v = 1.0 - (phi / PI);
        return Coordinates::new(u, v);
    }
}
