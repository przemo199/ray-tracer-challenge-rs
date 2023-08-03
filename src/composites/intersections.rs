use crate::composites::Intersection;
use core::ops::Index;

#[derive(Clone, Debug)]
pub struct Intersections<'a> {
    pub intersections: Vec<Intersection<'a>>,
}

impl<'a> Intersections<'a> {
    pub const fn new() -> Intersections<'a> {
        return Intersections {
            intersections: Vec::new(),
        };
    }

    pub fn with<const T: usize>(elements: [Intersection<'a>; T]) -> Intersections<'a> {
        return Intersections {
            intersections: Vec::from(elements),
        };
    }

    pub fn add(&mut self, item: Intersection<'a>) {
        self.intersections.push(item);
    }

    pub fn add_all(&mut self, intersections: Intersections<'a>) {
        self.intersections.extend(intersections.intersections);
    }

    pub fn len(&self) -> usize {
        return self.intersections.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.intersections.is_empty();
    }

    pub fn hit(&self) -> Option<&Intersection> {
        let mut maybe_hit = None;
        let mut hit_distance = f64::INFINITY;
        for intersection in &self.intersections {
            if intersection.is_within_distance(hit_distance) {
                maybe_hit = Some(intersection);
                hit_distance = intersection.distance;
            }
        }
        return maybe_hit;
    }

    pub fn sort_by_distance(&mut self) {
        self.intersections
            .sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    }

    pub fn into_option(self) -> Option<Intersections<'a>> {
        return if self.is_empty() { None } else { Some(self) };
    }
}

impl<'a> Default for Intersections<'a> {
    fn default() -> Self {
        return Intersections::new();
    }
}

impl<'a> Index<usize> for Intersections<'a> {
    type Output = Intersection<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.intersections[index];
    }
}

impl<'a> PartialEq for Intersections<'a> {
    fn eq(&self, rhs: &Self) -> bool {
        return self.len() == rhs.len()
            && self.into_iter().all(|intersection| {
                return rhs.intersections.contains(intersection);
            });
    }
}

impl<'a> IntoIterator for &'a Intersections<'a> {
    type Item = &'a Intersection<'a>;
    type IntoIter = core::slice::Iter<'a, Intersection<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.intersections.iter();
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
        intersections.add(intersection_1.clone());
        intersections.add(intersection_2);
        assert_eq!(intersections.hit().unwrap(), &intersection_1);
    }

    #[test]
    fn hit_when_some_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(-1, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(1, boxed_shape.as_ref());
        intersections.add(intersection_1);
        intersections.add(intersection_2.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection_2);
    }

    #[test]
    fn hit_when_all_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection_1 = Intersection::new(-2, boxed_shape.as_ref());
        let intersection_2 = Intersection::new(-1, boxed_shape.as_ref());
        intersections.add(intersection_1);
        intersections.add(intersection_2);
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
        intersections.add(intersection_1);
        intersections.add(intersection_2);
        intersections.add(intersection_3);
        intersections.add(intersection_4.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection_4);
    }
}
