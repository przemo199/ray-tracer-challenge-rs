# ray tracer challenge

[![Tests](https://github.com/przemo199/ray-tracer-challenge-rs/actions/workflows/tests.yaml/badge.svg)](https://github.com/przemo199/ray-tracer-challenge-rs/actions/workflows/tests.yaml)

Implementation of ray tracer from Ray Tracer Challenge book by [Jamis Buck](mailto:jamis@jamisbuck.org) using Rust programming language.

## Features

- Parallelized rendering using [rayon](https://github.com/rayon-rs/rayon)
- No cloning, only borrowed values used in rendering pipeline
- Handling for multiple light sources
- Efficient matrix implementation using const generics
- Shapes and Patterns implemented using traits
- No ```unsafe``` code

## Usage

```
Usage: ray-tracer-challenge.exe [OPTIONS] <SCENE_PATH> <IMAGE_OUTPUT_PATH>

Arguments:
  <SCENE_PATH>
  <IMAGE_OUTPUT_PATH>

Options:
  -r, --rendering-mode <RENDERING_MODE>  [default: parallel] [possible values: serial, parallel]
  -h, --help                             Print help
  -V, --version                          Print version
```

## Examples

![](ray-tracer-cli/rendered_images/cover.png)

![](ray-tracer-cli/rendered_images/reflect_refract.png)

![](ray-tracer-cli/rendered_images/refraction.png)

![](ray-tracer-cli/rendered_images/metal.png)

![](ray-tracer-cli/rendered_images/cylinders.png)

![](ray-tracer-cli/rendered_images/table.png)

![](ray-tracer-cli/rendered_images/three_sphere_scene.png)

![](ray-tracer-cli/rendered_images/shadow_puppets.png)
