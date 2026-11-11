use core::any::Any;

pub trait DynPartialEq: Any + private::Sealed {
    fn as_any(&self) -> &dyn Any;

    /// What `==` on trait objects (`dyn Shape`, `dyn ColorProvider`) uses: `other` is this very
    /// object, or has the same type and equal contents.
    fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T: PartialEq + 'static> DynPartialEq for T {
    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        // The same object is equal without comparing every field: the common case when
        // intersections are matched against their own shapes. Same address alone doesn't prove it
        // (all zero-sized values share one; a value can sit at the start of another), but same
        // address *and* same type does. The type is only checked when the addresses match.
        let is_same_object = core::ptr::addr_eq(self, other) && other.is::<T>();
        return is_same_object || other.downcast_ref().is_some_and(|other: &T| self == other);
    }
}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for T where T: PartialEq {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Cube, Shape, Sphere};

    #[test]
    fn equal_values_of_the_same_type_are_equal() {
        assert!(Sphere::default().dyn_eq(Sphere::default().as_any()));
    }

    #[test]
    fn same_object_is_equal_without_comparing_fields() {
        // NaN != NaN, so field-by-field these spheres are unequal; only the identity check
        // can make an object equal to itself
        let mut sphere = Sphere::default();
        sphere.material.ambient = f64::NAN;
        let sphere: Box<dyn Shape> = Box::new(sphere);
        let copy: Box<dyn Shape> = Box::new(Sphere::default());
        assert!(sphere.as_ref() == sphere.as_ref());
        assert!(sphere.as_ref() != copy.as_ref());
        // through references to references, as when iterating a `Vec<&dyn Shape>`
        let shapes: Vec<&dyn Shape> = vec![sphere.as_ref()];
        assert!(shapes.iter().any(|shape| *shape == sphere.as_ref()));
        assert!(shapes.iter().any(|shape| shape == &sphere.as_ref()));
    }

    #[test]
    fn values_of_different_types_are_never_equal() {
        assert!(!Sphere::default().dyn_eq(Cube::default().as_any()));
        assert!(!1_i32.dyn_eq(&1_i64));
    }

    #[test]
    fn zero_sized_values_of_different_types_are_not_equal() {
        #[derive(PartialEq)]
        struct First;
        #[derive(PartialEq)]
        struct Second;
        // every boxed zero-sized value gets the same dangling address
        let (first, second): (Box<dyn DynPartialEq>, Box<dyn DynPartialEq>) =
            (Box::new(First), Box::new(Second));
        assert!(core::ptr::addr_eq(first.as_ref(), second.as_ref()));
        assert!(!first.dyn_eq(second.as_any()));
        assert!(first.dyn_eq(First.as_any()));
    }
}
