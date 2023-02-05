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
        let mut hit_item = None;
        let mut hit_t = f64::MAX;
        for item in self.intersections.iter() {
            if item.t < hit_t && item.t >= 0.0 {
                hit_item = Some(item);
                hit_t = item.t;
            }
        }
        return hit_item;
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
    use crate::shape::Shape;
    use super::*;
    use crate::sphere::Sphere;

    #[test]
    fn hit_when_all_intersections_positive() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(1.0, arc_sphere.clone());
        let intersection2 = Intersection::new(2.0, arc_sphere);
        intersections.add(intersection1.clone());
        intersections.add(intersection2);
        assert_eq!(intersections.hit().unwrap(), &intersection1);
    }

    #[test]
    fn hit_when_some_intersections_negative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-1.0, arc_sphere.clone());
        let intersection2 = Intersection::new(1.0, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection2);
    }

    #[test]
    fn hit_when_all_intersections_negative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(-2.0, arc_sphere.clone());
        let intersection2 = Intersection::new(-1.0, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2);
        assert_eq!(intersections.hit(), None);
    }

    #[test]
    fn hit_always_lowest_nonnegative() {
        let sphere = Sphere::default();
        let arc_sphere: Arc<dyn Shape> = Arc::new(sphere);
        let mut intersections = Intersections::new();
        let intersection1 = Intersection::new(5.0, arc_sphere.clone());
        let intersection2 = Intersection::new(7.0, arc_sphere.clone());
        let intersection3 = Intersection::new(-3.0, arc_sphere.clone());
        let intersection4 = Intersection::new(2.0, arc_sphere);
        intersections.add(intersection1);
        intersections.add(intersection2);
        intersections.add(intersection3);
        intersections.add(intersection4.clone());
        assert_eq!(intersections.hit().unwrap(), &intersection4);
    }
}
