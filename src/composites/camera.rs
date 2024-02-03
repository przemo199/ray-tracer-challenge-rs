use crate::composites::{Canvas, Ray, World};
use crate::consts::PROGRESS_TEMPLATE;
use crate::primitives::Point;
use crate::primitives::{transformations, Transformation};
use crate::utils::CoarseEq;
use core::fmt::{Display, Formatter};
use indicatif::{ParallelProgressIterator, ProgressIterator, ProgressStyle};
use rayon::iter::ParallelIterator;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    horizontal_size: u32,
    vertical_size: u32,
    field_of_view: f64,
    half_width: f64,
    half_height: f64,
    pixel_size: f64,
    pub transformation: Transformation,
}

impl Camera {
    pub fn new(horizontal_size: u32, vertical_size: u32, field_of_view: impl Into<f64>) -> Self {
        let field_of_view = field_of_view.into();
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
        return Self {
            horizontal_size,
            vertical_size,
            field_of_view,
            half_width,
            half_height,
            pixel_size,
            transformation: transformations::IDENTITY,
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
        let pixel = self.transformation.inverse() * Point::new(world_x, world_y, -1);
        let origin = self.transformation.inverse() * Point::ORIGIN;
        let direction = (pixel - origin).normalized();
        return Ray::new(origin, direction);
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut canvas = Canvas::new(self.horizontal_size, self.vertical_size);
        let style = ProgressStyle::with_template(PROGRESS_TEMPLATE)
            .expect("Failed to create ProgressStyle");
        canvas
            .pixels
            .iter_mut()
            .progress_with_style(style)
            .enumerate()
            .for_each(|(index, pixel)| {
                let x: u32 = index as u32 % canvas.width;
                let y: u32 = index as u32 / canvas.width;
                let ray = self.ray_for_pixel(x, y);
                *pixel = world.color_at(&ray);
            });
        return canvas;
    }

    pub fn render_parallel(&self, world: &World) -> Canvas {
        let mut canvas = Canvas::new(self.horizontal_size, self.vertical_size);
        let style =
            ProgressStyle::with_template(PROGRESS_TEMPLATE).expect("Failed to parse ProgressStyle");
        canvas
            .pixels
            .par_iter_mut()
            .progress_with_style(style)
            .enumerate()
            .for_each(|(index, pixel)| {
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
        return self.horizontal_size == rhs.horizontal_size
            && self.vertical_size == rhs.vertical_size
            && self.horizontal_size == rhs.horizontal_size
            && self.field_of_view.coarse_eq(rhs.field_of_view)
            && self.half_width.coarse_eq(rhs.half_width)
            && self.half_height.coarse_eq(rhs.half_height)
            && self.pixel_size.coarse_eq(rhs.pixel_size);
    }
}

impl Display for Camera {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        return formatter
            .debug_struct("Camera")
            .field("horizontal_size", &self.horizontal_size)
            .field("vertical_size", &self.vertical_size)
            .field("field_of_view", &self.field_of_view)
            .field("half_width", &self.half_width)
            .field("half_height", &self.half_height)
            .field("pixel_size", &self.pixel_size)
            .field("transformation", &self.transformation)
            .finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PI;
    use crate::primitives::{Color, Vector};

    #[test]
    fn constructing_camera() {
        let horizontal_size = 160;
        let vertical_size = 120;
        let field_of_view = PI / 2.0;
        let camera = Camera::new(horizontal_size, vertical_size, field_of_view);
        assert_eq!(camera.horizontal_size, horizontal_size);
        assert_eq!(camera.vertical_size, vertical_size);
        assert_eq!(camera.field_of_view, field_of_view);
        assert_eq!(camera.transformation, transformations::IDENTITY);
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
        assert_eq!(ray.origin, Point::ORIGIN);
        assert_eq!(ray.direction, Vector::BACKWARD);
    }

    #[test]
    fn ray_through_canvas_corner() {
        let camera = Camera::new(201, 101, PI / 2.0);
        let ray = camera.ray_for_pixel(0, 0);
        assert_eq!(ray.origin, Point::ORIGIN);
        assert_eq!(
            ray.direction,
            Vector::new(0.6651864261194508, 0.3325932130597254, -0.6685123582500481)
        );
    }

    #[test]
    fn ray_through_canvas_with_transformed_camera() {
        let mut camera = Camera::new(201, 101, PI / 2.0);
        camera.transformation =
            transformations::rotation_y(PI / 4.0) * transformations::translation(0, -2, 5);
        let ray = camera.ray_for_pixel(100, 50);
        assert_eq!(ray.origin, Point::new(0, 2, -5));
        assert_eq!(
            ray.direction,
            Vector::new(2.0_f64.sqrt() / 2.0, 0, -(2.0_f64.sqrt()) / 2.0)
        );
    }

    #[test]
    fn rendering_world_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Point::new(0, 0, -5);
        let to = Point::ORIGIN;
        let up = Vector::UP;
        camera.transformation = transformations::view_transform(from, to, up);
        let canvas = camera.render(&world);
        assert_eq!(
            canvas.get_pixel(5, 5),
            &Color::new(
                0.38066119308103435,
                0.47582649135129296,
                0.28549589481077575
            )
        );
    }

    #[test]
    fn rendering_world_in_parallel_with_camera() {
        let world = World::default();
        let mut camera = Camera::new(11, 11, PI / 2.0);
        let from = Point::new(0, 0, -5);
        let to = Point::ORIGIN;
        let up = Vector::UP;
        camera.transformation = transformations::view_transform(from, to, up);
        let canvas = camera.render_parallel(&world);
        assert_eq!(
            canvas.get_pixel(5, 5),
            &Color::new(
                0.38066119308103435,
                0.47582649135129296,
                0.28549589481077575
            )
        );
    }
}
