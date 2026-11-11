use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Color, Coordinates};
use core::fmt::Debug;

/// A 2D pattern: a color for each `(u, v)` texture coordinate in `0..=1`. Combined with a
/// [`Mapping`](crate::patterns::Mapping) in a [`TextureMap`](crate::patterns::TextureMap), any
/// UV pattern works on any shape the mapping supports.
pub trait UvPattern: Debug + Send + Sync + DynPartialEq + CloneUvPattern {
    fn color_at(&self, coordinates: &Coordinates) -> Color;
}

/// Makes `Box<dyn UvPattern>` cloneable: `Clone` itself can't be called through `dyn`.
/// Implemented automatically for every `Clone` UV pattern.
pub trait CloneUvPattern {
    fn clone_box(&self) -> Box<dyn UvPattern>;
}

impl<T: UvPattern + Clone + 'static> CloneUvPattern for T {
    fn clone_box(&self) -> Box<dyn UvPattern> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<dyn UvPattern> {
    fn clone(&self) -> Self {
        return self.clone_box();
    }
}

impl PartialEq for dyn UvPattern {
    fn eq(&self, other: &Self) -> bool {
        return self.dyn_eq(DynPartialEq::as_any(other));
    }
}
