use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Color, Point};
use crate::shapes::Shape;
use core::fmt::{Debug, Display};

pub trait ColorProvider: Debug + Display + Send + Sync + DynPartialEq {
    fn color_at_shape(&self, shape: &dyn Shape, point: &Point) -> Color;
}

impl PartialEq for dyn ColorProvider {
    fn eq(&self, other: &Self) -> bool {
        return self.dyn_eq(DynPartialEq::as_any(other));
    }
}

impl PartialEq<&Self> for Box<dyn ColorProvider> {
    fn eq(&self, other: &&Self) -> bool {
        return self == *other;
    }
}
