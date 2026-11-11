use core::any::Any;

pub trait DynPartialEq: Any + private::Sealed {
    fn as_any(&self) -> &dyn Any;

    fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T: PartialEq + 'static> DynPartialEq for T {
    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        return other
            .downcast_ref()
            .map_or(false, |element| self == element);
    }
}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for T where T: PartialEq {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Cube, Sphere};

    #[test]
    fn equal_values_of_the_same_type_are_equal() {
        assert!(Sphere::default().dyn_eq(Sphere::default().as_any()));
    }

    #[test]
    fn values_of_different_types_are_never_equal() {
        assert!(!Sphere::default().dyn_eq(Cube::default().as_any()));
        assert!(!1_i32.dyn_eq(&1_i64));
    }
}
