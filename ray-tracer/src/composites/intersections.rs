use crate::composites::Intersection;
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

#[derive(Clone, Debug)]
pub struct Intersections<'intersections>(Vec<Intersection<'intersections>>);

impl<'intersections> Intersections<'intersections> {
    pub const fn new() -> Intersections<'intersections> {
        return Intersections(Vec::new());
    }

    pub fn hit(&self) -> Option<&Intersection> {
        return self
            .iter()
            .filter(|intersection| intersection.distance >= 0.0)
            .min();
    }
}

impl Default for Intersections<'_> {
    fn default() -> Self {
        return Intersections::new();
    }
}

impl<'intersections> Deref for Intersections<'intersections> {
    type Target = Vec<Intersection<'intersections>>;

    fn deref(&self) -> &Self::Target {
        return &(self.0);
    }
}

impl DerefMut for Intersections<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl PartialEq for Intersections<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.len() == rhs.len()
                && self.into_iter().all(|intersection| {
                    return rhs.contains(intersection);
                });
    }
}

impl<'intersections> From<Intersections<'intersections>> for Vec<Intersection<'intersections>> {
    fn from(value: Intersections<'intersections>) -> Self {
        return value.0;
    }
}

impl<'intersections> IntoIterator for &'intersections Intersections<'intersections> {
    type Item = &'intersections Intersection<'intersections>;
    type IntoIter = Iter<'intersections, Intersection<'intersections>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.0.iter();
    }
}

impl<'intersection> From<Vec<Intersection<'intersection>>> for Intersections<'intersection> {
    fn from(value: Vec<Intersection<'intersection>>) -> Self {
        return Self(value);
    }
}

impl<'intersection, const SIZE: usize> From<[Intersection<'intersection>; SIZE]>
    for Intersections<'intersection>
{
    fn from(value: [Intersection<'intersection>; SIZE]) -> Self {
        return Self(value.to_vec());
    }
}

impl<'intersection> From<Intersection<'intersection>> for Intersections<'intersection> {
    fn from(value: Intersection<'intersection>) -> Self {
        return [value].into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Shape, Sphere};

    #[test]
    fn hit_when_all_intersections_positive() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(1, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(2, boxed_shape.as_ref());
        intersections.push(intersection_1.clone());
        intersections.push(intersection_2);
        assert_eq!(intersections.hit(), Some(&intersection_1));
    }

    #[test]
    fn hit_when_some_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(-1, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(1, boxed_shape.as_ref());
        intersections.push(intersection_1);
        intersections.push(intersection_2.clone());
        assert_eq!(intersections.hit(), Some(&intersection_2));
    }

    #[test]
    fn hit_when_all_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(-2, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(-1, boxed_shape.as_ref());
        intersections.push(intersection_1);
        intersections.push(intersection_2);
        assert_eq!(intersections.hit(), None);
    }

    #[test]
    fn hit_always_lowest_non_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(5, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(7, boxed_shape.as_ref());
        let intersection_3 = Intersection::new(-3, boxed_shape.as_ref());
        let intersection_4 = Intersection::new(2, boxed_shape.as_ref());
        intersections.push(intersection_1);
        intersections.push(intersection_2);
        intersections.push(intersection_3);
        intersections.push(intersection_4.clone());
        assert_eq!(intersections.hit(), Some(&intersection_4));
    }
}
