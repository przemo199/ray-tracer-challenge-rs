use crate::composites::{Intersections, Material, Ray};
use crate::dyn_partial_eq::DynPartialEq;
use crate::primitives::{Point, Transformation, Vector};
use core::fmt::Debug;

pub trait Transform {
    fn transformation(&self) -> Transformation;

    fn set_transformation(&mut self, transformation: Transformation);

    fn transformation_inverse(&self) -> Transformation;

    /// Transforms object-space normals to world space: the transposed inverse. Shapes store it
    /// (see [`impl_shape_transform_for`](crate::impl_shape_transform_for)); others compute it.
    fn normal_transformation(&self) -> Transformation {
        return self.transformation_inverse().transpose();
    }
}

#[macro_export]
macro_rules! impl_transform_for {
    ($type:ty) => {
        impl $crate::shapes::Transform for $type {
            fn transformation(&self) -> Transformation {
                return self.transformation_inverse.inverse();
            }

            fn set_transformation(&mut self, transformation: Transformation) {
                self.transformation_inverse = transformation.inverse();
            }

            fn transformation_inverse(&self) -> Transformation {
                return self.transformation_inverse;
            }
        }
    };
}

/// Like [`impl_transform_for`], for shapes: the type also stores `normal_transformation` (the
/// transposed inverse), so `Shape::normal_at` doesn't recompute it for every hit.
#[macro_export]
macro_rules! impl_shape_transform_for {
    ($type:ty) => {
        impl $crate::shapes::Transform for $type {
            fn transformation(&self) -> Transformation {
                return self.transformation_inverse.inverse();
            }

            fn set_transformation(&mut self, transformation: Transformation) {
                self.transformation_inverse = transformation.inverse();
                self.normal_transformation = self.transformation_inverse.transpose();
            }

            fn transformation_inverse(&self) -> Transformation {
                return self.transformation_inverse;
            }

            fn normal_transformation(&self) -> Transformation {
                return self.normal_transformation;
            }
        }
    };
}

pub trait Intersect {
    fn local_intersect<'shape>(&'shape self, ray: &Ray, intersections: &mut Intersections<'shape>);
}

pub trait Shape: Debug + Send + Sync + Transform + Intersect + DynPartialEq {
    #[inline]
    fn normal_at(&self, point: Point) -> Vector {
        let local_point = self.transformation_inverse() * point;
        let local_normal = self.local_normal_at(local_point);
        let world_normal = self.normal_transformation() * local_normal;
        return world_normal.normalized();
    }

    fn local_normal_at(&self, point: Point) -> Vector;

    fn material(&self) -> &Material;
}

/// Equal if it is the same shape object, or a shape of the same type with equal contents
/// (see [`DynPartialEq::dyn_eq`]).
impl PartialEq for dyn Shape {
    fn eq(&self, other: &Self) -> bool {
        return self.dyn_eq(other);
    }
}

impl PartialEq<&Self> for Box<dyn Shape> {
    fn eq(&self, other: &&Self) -> bool {
        return self == *other;
    }
}

#[cfg(test)]
mod tests {
    use crate::composites::Material;
    use crate::primitives::Color;
    use crate::shapes::Sphere;

    use super::*;

    #[test]
    fn default_material() {
        let shape = Sphere::default();
        assert_eq!(shape.material(), &Material::default());
    }

    #[test]
    fn assigning_material() {
        let mut shape = Sphere::default();
        let material = Material {
            color: Color::new(0.8, 1, 0.6),
            ..Material::default()
        };
        shape.material = material.clone();
        assert_eq!(shape.material(), &material);
    }
}
