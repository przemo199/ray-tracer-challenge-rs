//! Ray–shape intersection, surface normals and hit selection.
//! Shapes are called through `&dyn Shape`, as `World` does during a render.

mod common;

use common::{BATCH, Rng, fast_config, map_into};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ray_tracer::composites::{Intersection, Intersections, Material, Ray};
use ray_tracer::primitives::{Point, Transformation, Vector};
use ray_tracer::shapes::{Cone, Cube, Cylinder, Plane, Shape, Sphere, Triangle};
use std::hint::black_box;

/// Name, shape, and a sampler of points inside it. All but the plane fit within radius 1.8.
type Case = (&'static str, Box<dyn Shape>, fn(&mut Rng) -> Point);

fn shapes() -> Vec<Case> {
    let material = Material::default;
    return vec![
        ("sphere", Box::new(Sphere::default()), |rng| {
            rng.point_in_ball(Point::ORIGIN, 0.5)
        }),
        ("plane", Box::new(Plane::default()), |rng| {
            Point::new(rng.range(-2.0, 2.0), 0, rng.range(-2.0, 2.0))
        }),
        ("cube", Box::new(Cube::default()), |rng| {
            rng.point_in_ball(Point::ORIGIN, 0.5)
        }),
        (
            "cylinder",
            Box::new(Cylinder::new(
                material(),
                Transformation::IDENTITY,
                -1,
                1,
                true,
            )),
            |rng| rng.point_in_ball(Point::ORIGIN, 0.5),
        ),
        (
            "cone",
            Box::new(Cone::new(material(), Transformation::IDENTITY, -1, 0, true)),
            |rng| rng.point_in_ball(Point::new(0, -0.6, 0), 0.3),
        ),
        (
            "triangle",
            Box::new(Triangle::new(
                Point::new(0, 1, 0),
                Point::new(-1, 0, 0),
                Point::new(1, 0, 0),
            )),
            |rng| Point::new(rng.range(-0.4, 0.4), rng.range(0.05, 0.5), 0),
        ),
    ];
}

/// Rays from 5 units away through `target`; `target` 3 units off-center makes them miss.
fn rays(rng: &mut Rng, target: impl Fn(&mut Rng, Vector) -> Point) -> Vec<Ray> {
    return rng.batch(|rng| {
        let outward = rng.unit_vector();
        let origin = Point::ORIGIN + outward * 5.0;
        return Ray::new(origin, (target(rng, outward) - origin).normalized());
    });
}

fn intersect_all<'shape>(shape: &'shape dyn Shape, rays: &[Ray], xs: &mut Intersections<'shape>) {
    for ray in black_box(rays) {
        xs.clear();
        ray.intersect(shape, xs);
        black_box(&*xs);
    }
}

fn bench_intersect(criterion: &mut Criterion) {
    let mut rng = Rng::new(10);
    let shapes = shapes();
    let mut group = criterion.benchmark_group("intersect");
    group.throughput(Throughput::Elements(BATCH as u64));
    for (name, shape, inside) in &shapes {
        let hits = rays(&mut rng, |rng, _| inside(rng));
        let misses = rays(&mut rng, |rng, outward| {
            let random = rng.unit_vector();
            let perpendicular = (random - outward * random.dot(&outward)).normalized();
            return Point::ORIGIN + perpendicular * 3.0;
        });
        for (kind, rays) in [("hit", hits), ("miss", misses)] {
            if *name == "plane" && kind == "miss" {
                continue; // an infinite plane is hit by every non-parallel ray
            }
            // make sure the benchmark measures what its name says
            let mut xs = Intersections::new();
            for ray in &rays {
                xs.clear();
                ray.intersect(shape.as_ref(), &mut xs);
                assert_eq!(xs.hit().is_some(), kind == "hit", "{name}/{kind}: {ray:?}");
            }
            group.bench_function(BenchmarkId::new(*name, kind), |bencher| {
                bencher.iter(|| intersect_all(shape.as_ref(), &rays, &mut xs));
            });
        }
    }
    group.finish();
}

fn bench_normal_at(criterion: &mut Criterion) {
    let mut rng = Rng::new(11);
    let sphere_points = rng.batch(|rng| Point::from(rng.unit_vector()));
    let cube_points = rng.batch(|rng| {
        // project onto the cube's surface along the largest component
        let vector = rng.unit_vector();
        let abs = vector.abs();
        return Point::from(vector / abs.x.max(abs.y).max(abs.z));
    });
    let shapes: [(&str, Box<dyn Shape>, Vec<Point>); 2] = [
        ("sphere", Box::new(Sphere::default()), sphere_points),
        ("cube", Box::new(Cube::default()), cube_points),
    ];
    let mut normals = vec![Vector::ZERO; BATCH];
    let mut group = criterion.benchmark_group("normal_at");
    group.throughput(Throughput::Elements(BATCH as u64));
    for (name, shape, points) in &shapes {
        group.bench_function(*name, |bencher| {
            bencher.iter(|| map_into(points, &mut normals, |&point| shape.normal_at(point)));
        });
    }
    group.finish();
}

fn bench_hit(criterion: &mut Criterion) {
    let mut rng = Rng::new(12);
    let sphere = Sphere::default();
    // 8 intersections per ray, some behind the origin (negative), as shadows and recursion produce
    let lists: Vec<Intersections> = (0..BATCH / 8)
        .map(|_| {
            let mut xs = Intersections::new();
            xs.extend((0..8).map(|_| Intersection::new(rng.range(-5.0, 10.0), &sphere)));
            return xs;
        })
        .collect();
    let mut group = criterion.benchmark_group("intersections");
    group.throughput(Throughput::Elements(lists.len() as u64));
    group.bench_function("hit/8", |bencher| {
        bencher.iter(|| {
            for xs in black_box(&lists) {
                black_box(xs.hit());
            }
        });
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = fast_config();
    targets = bench_intersect, bench_normal_at, bench_hit
}
criterion_main!(benches);
