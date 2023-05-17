use std::ops::Index;

use crate::intersection::Intersection;

#[derive(Clone, Debug)]
pub struct Intersections<'a> {
    pub intersections: Vec<Intersection<'a>>,
}

impl<'a> Intersections<'a> {
    pub fn new() -> Intersections<'a> {
        return Intersections { intersections: Vec::new() };
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

    pub fn hit(&self) -> Option<&Intersection> {
        let mut maybe_hit = None;
        let mut hit_distance = f64::MAX;
        for intersection in &self.intersections {
            if intersection.distance < hit_distance && intersection.distance >= 0.0 {
                maybe_hit = Some(intersection);
                hit_distance = intersection.distance;
            }
        }
        return maybe_hit;
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
        return self.len() == rhs.len() && self.into_iter().all(|intersection| {
            return rhs.intersections.contains(&intersection);
        });
    }
}

impl<'a> IntoIterator for &'a Intersections<'a> {
    type Item = &'a Intersection<'a>;
    type IntoIter = std::slice::Iter<'a, Intersection<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.intersections.iter();
    }
}

#[cfg(test)]
mod tests {
    use crate::shapes::{Shape, Sphere};

    use super::*;

    #[test]
    fn hit_when_all_intersections_positive() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(1, boxed_shape.as_ref());
        let intersection2 = Intersection::new(2, boxed_shape.as_ref());
        intersections.add(intersection1.clone());
        intersections.add(intersection2);
        assert_eq!(intersections.hit().unwrap(), &intersection1);
    }

    #[test]
    fn hit_when_some_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-1, boxed_shape.as_ref());
        let intersection2 = Intersection::new(1, boxed_shape.as_ref());
        intersections.add(intersection1);
        intersections.add(intersection2.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection2);
    }

    #[test]
    fn hit_when_all_intersections_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-2, boxed_shape.as_ref());
        let intersection2 = Intersection::new(-1, boxed_shape.as_ref());
        intersections.add(intersection1);
        intersections.add(intersection2);
        assert_eq!(intersections.hit(), None);
    }

    #[test]
    fn hit_always_lowest_non_negative() {
        let sphere = Sphere::default();
        let boxed_shape: Box<dyn Shape> = Box::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(5, boxed_shape.as_ref());
        let intersection2 = Intersection::new(7, boxed_shape.as_ref());
        let intersection3 = Intersection::new(-3, boxed_shape.as_ref());
        let intersection4 = Intersection::new(2, boxed_shape.as_ref());
        intersections.add(intersection1);
        intersections.add(intersection2);
        intersections.add(intersection3);
        intersections.add(intersection4.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection4);
    }
}
