use crate::composites::Intersection;
use crate::shapes::Shape;
use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

/// The intersections of a ray with the scene, plus a reusable scratch buffer for shading.
#[derive(Clone, Debug)]
pub struct Intersections<'intersections> {
    hits: Vec<Intersection<'intersections>>,
    /// Shapes a ray is inside; see [`Intersections::containers`].
    containers: RefCell<Vec<&'intersections dyn Shape>>,
}

impl<'intersections> Intersections<'intersections> {
    pub const fn new() -> Intersections<'intersections> {
        return Self::from_hits(Vec::new());
    }

    const fn from_hits(hits: Vec<Intersection<'intersections>>) -> Self {
        return Self {
            hits,
            containers: RefCell::new(Vec::new()),
        };
    }

    /// Scratch list used by [`Intersection::prepare_computations`] to track which shapes a ray is
    /// inside. Kept here so one allocation is reused for every hit instead of one per hit.
    pub(crate) fn containers(&self) -> RefMut<'_, Vec<&'intersections dyn Shape>> {
        return self.containers.borrow_mut();
    }

    /// The closest intersection in front of the ray origin (non-negative distance).
    pub fn hit(&self) -> Option<&Intersection<'_>> {
        return self
            .iter()
            .filter(|intersection| intersection.distance >= 0.0)
            .min_by(|a, b| a.distance.total_cmp(&b.distance));
    }

    /// Sorts by distance, nearest first. Stable, so intersections at equal distances keep their
    /// order (which matters for tracking the refractive indices of overlapping shapes).
    pub fn sort_by_distance(&mut self) {
        self.hits.sort_by(|a, b| a.distance.total_cmp(&b.distance));
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
        return &self.hits;
    }
}

impl DerefMut for Intersections<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.hits;
    }
}

impl PartialEq for Intersections<'_> {
    fn eq(&self, other: &Self) -> bool {
        return std::ptr::eq(self, other)
            || self.len() == other.len()
                && self.into_iter().all(|intersection| {
                    return other.contains(intersection);
                });
    }
}

impl<'intersections> From<Intersections<'intersections>> for Vec<Intersection<'intersections>> {
    fn from(value: Intersections<'intersections>) -> Self {
        return value.hits;
    }
}

impl<'list, 'intersections> IntoIterator for &'list Intersections<'intersections> {
    type Item = &'list Intersection<'intersections>;
    type IntoIter = Iter<'list, Intersection<'intersections>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.hits.iter();
    }
}

impl<'intersection> From<Vec<Intersection<'intersection>>> for Intersections<'intersection> {
    fn from(value: Vec<Intersection<'intersection>>) -> Self {
        return Self::from_hits(value);
    }
}

impl<'intersection, const SIZE: usize> From<[Intersection<'intersection>; SIZE]>
    for Intersections<'intersection>
{
    fn from(value: [Intersection<'intersection>; SIZE]) -> Self {
        return Self::from_hits(value.to_vec());
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
