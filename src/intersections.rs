use std::ops::Index;

use crate::intersection::Intersection;

#[derive(Clone, Debug)]
pub struct Intersections {
    pub intersections: Vec<Intersection>,
}

impl Intersections {
    pub fn new() -> Intersections {
        return Intersections { intersections: Vec::new() };
    }

    pub fn add(&mut self, item: Intersection) {
        self.intersections.push(item);
    }

    pub fn add_all(&mut self, intersections: Intersections) {
        self.intersections.extend(intersections.intersections);
    }

    pub fn len(&self) -> usize {
        return self.intersections.len();
    }

    pub fn hit(&self) -> Option<&Intersection> {
        let mut maybe_hit = None;
        let mut hit_t = f64::MAX;
        for intersection in self.intersections.iter() {
            if intersection.distance < hit_t && intersection.distance >= 0.0 {
                maybe_hit = Some(intersection);
                hit_t = intersection.distance;
            }
        }
        return maybe_hit;
    }
}

impl Index<usize> for Intersections {
    type Output = Intersection;

    fn index(&self, index: usize) -> &Intersection {
        return &self.intersections[index];
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::shapes::{Shape, Sphere};

    use super::*;

    #[test]
    fn hit_when_all_intersections_positive() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(1, arc_sphere.clone());
        let intersection2 = Intersection::new(2, arc_sphere);
        intersections.add(intersection1.clone());
        intersections.add(intersection2);
        assert_eq!(intersections.hit().unwrap(), &intersection1);
    }

    #[test]
    fn hit_when_some_intersections_negative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-1, arc_sphere.clone());
        let intersection2 = Intersection::new(1, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection2);
    }

    #[test]
    fn hit_when_all_intersections_negative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-2, arc_sphere.clone());
        let intersection2 = Intersection::new(-1, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2);
        assert_eq!(intersections.hit(), None);
    }

    #[test]
    fn hit_always_lowest_non_negative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(5, arc_sphere.clone());
        let intersection2 = Intersection::new(7, arc_sphere.clone());
        let intersection3 = Intersection::new(-3, arc_sphere.clone());
        let intersection4 = Intersection::new(2, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2);
        intersections.add(intersection3);
        intersections.add(intersection4.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection4);
    }
}
