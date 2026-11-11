use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Color, Point};
use crate::shapes::Shape;
use core::fmt::{Debug, Display};

pub trait ColorProvider: Debug + Display + Send + Sync + DynPartialEq + CloneColorProvider {
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

/// Makes `Box<dyn ColorProvider>` cloneable: `Clone` itself can't be called through `dyn`.
/// Implemented automatically for every `Clone` color provider.
pub trait CloneColorProvider {
    fn clone_box(&self) -> Box<dyn ColorProvider>;
}

impl<T: ColorProvider + Clone + 'static> CloneColorProvider for T {
    fn clone_box(&self) -> Box<dyn ColorProvider> {
        return Box::new(self.clone());
    }
}

impl Clone for Box<dyn ColorProvider> {
    fn clone(&self) -> Self {
        return self.clone_box();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::StripePattern;
    use crate::primitives::transformations;
    use crate::shapes::{Sphere, Transform};

    #[test]
    fn boxed_color_providers_clone_with_their_state() {
        let mut stripes = StripePattern::new(Color::WHITE, Color::BLACK);
        stripes.set_transformation(transformations::scaling(0.5, 1, 1));
        let original: Box<dyn ColorProvider> = Box::new(stripes);
        let copy = original.clone();
        assert!(copy == original);
        // an independent object, not a shared one
        assert!(!core::ptr::addr_eq(copy.as_ref(), original.as_ref()));
        let point = Point::new(0.75, 0, 0);
        let shape = Sphere::default();
        assert_eq!(
            copy.color_at_shape(&shape, &point),
            original.color_at_shape(&shape, &point)
        );
    }
}
