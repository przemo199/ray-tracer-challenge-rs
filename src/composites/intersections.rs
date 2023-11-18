use crate::composites::Intersection;
use core::ops::{Deref, DerefMut};
use core::slice::Iter;
use crate::consts::MAX;

#[derive(Clone, Debug)]
pub struct Intersections<'intersections> {
    pub intersections: Vec<Intersection<'intersections>>,
}

impl<'intersections> Intersections<'intersections> {
    pub const fn new() -> Intersections<'intersections> {
        return Intersections {
            intersections: Vec::new(),
        };
    }

    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    pub fn hit(&self) -> Option<&Intersection> {
        let mut maybe_hit = None;
        let mut hit_distance = MAX;
        for intersection in self {
            if intersection.is_within_distance(hit_distance) {
                maybe_hit = Some(intersection);
                hit_distance = intersection.distance;
            }
        }
        return maybe_hit;
    }

    pub fn sort_by_distance(&mut self) {
        self.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    }

    pub fn into_option(self) -> Option<Intersections<'intersections>> {
        return if self.is_empty() { None } else { Some(self) };
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
        return &(self.intersections);
    }
}

impl<'intersections> DerefMut for Intersections<'intersections> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut (self.intersections);
    }
}

impl PartialEq for Intersections<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        return self.len() == rhs.len()
            && self.into_iter().all(|intersection| {
                return rhs.contains(intersection);
            });
    }
}

impl<'intersections> IntoIterator for &'intersections Intersections<'intersections> {
    type Item = &'intersections Intersection<'intersections>;
    type IntoIter = Iter<'intersections, Intersection<'intersections>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.intersections.iter();
    }
}

impl<'intersection> From<Vec<Intersection<'intersection>>> for Intersections<'intersection> {
    fn from(value: Vec<Intersection<'intersection>>) -> Self {
        return Self {
            intersections: value,
        };
    }
}

impl<'intersection, const SIZE: usize> From<[Intersection<'intersection>; SIZE]>
    for Intersections<'intersection>
{
    fn from(value: [Intersection<'intersection>; SIZE]) -> Self {
        return Self {
            intersections: value.to_vec(),
        };
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
        assert_eq!(intersections.hit().unwrap(), &intersection_1);
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
        assert_eq!(intersections.hit().unwrap(), &intersection_2);
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
        assert_eq!(intersections.hit().unwrap(), &intersection_4);
    }
}
