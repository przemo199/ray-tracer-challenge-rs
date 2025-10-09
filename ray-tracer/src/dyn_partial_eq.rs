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
