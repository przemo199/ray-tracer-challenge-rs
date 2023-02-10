use rayon::prelude::*;
use crate::canvas::Canvas;
use crate::matrix::Matrix;
use crate::ray::Ray;
use crate::transformations::Transformations;
use crate::tuple::{Tuple, TupleTrait};
use crate::utils::CloseEnough;
use crate::world::World;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub horizontal_size: u32,
    pub vertical_size: u32,
    pub field_of_view: f64,
    pub half_width: f64,
    pub half_height: f64,
    pub pixel_size: f64,
    pub transformation: Matrix<4>,
}

impl Camera {
    pub fn new(horizontal_size: u32, vertical_size: u32, field_of_view: f64) -> Camera {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = horizontal_size as f64 / vertical_size as f64;
        let half_width: f64;
        let half_height: f64;
        if aspect >= 1.0 {
            half_width = half_view;
            half_height = half_view / aspect;
        } else {
            half_width = half_view * aspect;
            half_height = half_view;
        }
        let pixel_size = (half_width * 2.0) / (horizontal_size as f64);
        return Camera {
            horizontal_size,
            vertical_size,
            field_of_view,
            half_width,
            half_height,
            pixel_size,
            transformation: Transformations::identity(),
        };
    }

    pub fn ray_for_pixel(&self, pixel_x: u32, pixel_y: u32) -> Ray {
        // the offset from the edge of the canvas to the pixel's center
        let offset_x = (pixel_x as f64 + 0.5) * self.pixel_size;
        let offset_y = (pixel_y as f64 + 0.5) * self.pixel_size;

        // the untransformed coordinates of the pixel in world space
        // (remember that the camera looks toward -z, so +x is to the *left*)
        let world_x = self.half_width - offset_x;
        let world_y = self.half_height - offset_y;

        // using the camera matrix, transform the canvas point and the origin
        // and then compute the ray's direction vector
        // (remember that the canvas is at z = -1)
        let pixel = self.transformation.inverse() * Tuple::point(world_x, world_y, -1.0);
        let origin = self.transformation.inverse() * Tuple::point(0.0, 0.0, 0.0);
        let direction = (pixel - origin).normalize();
        return Ray::new(origin, direction);
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut canvas = Canvas::new(self.horizontal_size, self.vertical_size);
        canvas.pixels.iter_mut().enumerate().for_each(|(index, pixel)| {
            let x: u32 = index as u32 % canvas.width;
            let y: u32 = index as u32 / canvas.width;
            let ray = self.ray_for_pixel(x, y);
            *pixel = world.color_at(&ray);
        });
        return canvas;
    }

    pub fn render_parallel(&self, world: &World) -> Canvas {
        let mut canvas = Canvas::new(self.horizontal_size, self.vertical_size);
        canvas.pixels.par_iter_mut().enumerate().for_each(|(index, pixel)| {
            let x: u32 = index as u32 % canvas.width;
            let y: u32 = index as u32 / canvas.width;
            let ray = self.ray_for_pixel(x, y);
            *pixel = world.color_at(&ray);
        });
        return canvas;
    }
}

impl PartialEq for Camera {
    fn eq(&self, rhs: &Self) -> bool {
        return self.horizontal_size == rhs.horizontal_size &&
            self.vertical_size == rhs.vertical_size &&
            self.horizontal_size == rhs.horizontal_size &&
            self.field_of_view.close_enough(rhs.field_of_view) &&
            self.half_width.close_enough(rhs.half_width) &&
            self.half_height.close_enough(rhs.half_height) &&
            self.pixel_size.close_enough(rhs.pixel_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::consts::PI;

    #[test]
    fn constructing_camera() {
        let horizontal_size = 160;
        let vertical_size = 120;
        let field_of_view = PI / 2.0;
        let camera = Camera::new(horizontal_size, vertical_size, field_of_view);
        assert_eq!(camera.horizontal_size, horizontal_size);
        assert_eq!(camera.vertical_size, vertical_size);
        assert_eq!(camera.field_of_view, field_of_view);
        assert_eq!(camera.transformation, Transformations::identity());
    }

    #[test]
    fn pixel_size_for_horizontal_canvas() {
        let camera = Camera::new(200, 125, PI / 2.0);
        assert_eq!(camera.pixel_size, 0.009999999999999998);
    }

    #[test]
    fn pixel_size_for_vertical_canvas() {
        let camera = Camera::new(125, 200, PI / 2.0);
        assert_eq!(camera.pixel_size, 0.009999999999999998);
    }

    #[test]
    fn ray_through_canvas_center() {
        let camera = Camera::new(201, 101, PI / 2.0);
        let ray = camera.ray_for_pixel(100, 50);
        assert_eq!(ray.origin, Tuple::point(0.0, 0.0, 0.0));
        assert_eq!(ray.direction, Tuple::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn ray_through_canvas_corner() {
        let camera = Camera::new(201, 101, PI / 2.0);
        let ray = camera.ray_for_pixel(0, 0);
        assert_eq!(ray.origin, Tuple::point(0.0, 0.0, 0.0));
        assert_eq!(ray.direction, Tuple::vector(0.6651864261194508, 0.3325932130597254, -0.6685123582500481));
    }

    #[test]
    fn ray_through_canvas_with_transformed_camera() {
        let mut camera = Camera::new(201, 101, PI / 2.0);
        camera.transformation = Transformations::rotation_y(PI / 4.0) * Transformations::translation(0.0, -2.0, 5.0);
        let ray = camera.ray_for_pixel(100, 50);
        assert_eq!(ray.origin, Tuple::point(0.0, 2.0, -5.0));
        assert_eq!(ray.direction, Tuple::vector(2.0_f64.sqrt() / 2.0, 0.0, -(2.0_f64.sqrt()) / 2.0));
    }

    #[test]
    fn rendering_world_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Tuple::point(0.0, 0.0, -5.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        camera.transformation = Transformations::view_transform(from, to, up);
        let canvas = camera.render(&world);
        assert_eq!(canvas.get_pixel(5, 5), &Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }

    #[test]
    fn rendering_world_in_parallel_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Tuple::point(0.0, 0.0, -5.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        camera.transformation = Transformations::view_transform(from, to, up);
        let canvas = camera.render_parallel(&world);
        assert_eq!(canvas.get_pixel(5, 5), &Color::new(0.38066119308103435, 0.47582649135129296, 0.28549589481077575));
    }
}
