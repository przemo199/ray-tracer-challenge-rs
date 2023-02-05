use crate::matrix::Matrix;
use crate::shape::Shape;
use crate::transformations::Transformations;

#[derive(Debug)]
pub struct Group {
    pub shapes: Vec<Box<dyn Shape>>,
    pub transformation: Matrix<4>,
}

impl Group {
    pub fn new() -> Group {
        return Group { shapes: Vec::new(), transformation: Transformations::identity() };
    }
}

impl Default for Group {
    fn default() -> Self {
        return Group::new();
    }
}

#[cfg(test)]
mod tests {
    use crate::group::Group;
    use crate::transformations::Transformations;

    #[test]
    fn creating_new_group() {
        let group = Group::new();
        assert_eq!(group.transformation, Transformations::identity());
        assert_eq!(group.shapes.len(), 0);
    }
}
