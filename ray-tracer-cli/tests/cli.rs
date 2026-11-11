//! End-to-end tests that run the `ray-tracer-cli` binary on small scene files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SCENE: &str = "
- add: camera
  width: 24
  height: 16
  field-of-view: 1.0
  from: [0, 1.5, -5]
  to: [0, 1, 0]
  up: [0, 1, 0]
- add: light
  at: [-10, 10, -10]
  intensity: [1, 1, 1]
- add: plane
- add: sphere
  transform:
    - [translate, 0, 1, 0]
";

/// A per-test directory inside `target/` (see AGENTS.md).
fn directory(test: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cli")
        .join(test);
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    return directory;
}

fn write_scene(directory: &Path, yaml: &str) -> PathBuf {
    let path = directory.join("scene.yaml");
    fs::write(&path, yaml).unwrap();
    return path;
}

fn run(args: &[&std::ffi::OsStr]) -> Output {
    return Command::new(env!("CARGO_BIN_EXE_ray-tracer-cli"))
        .args(args)
        .output()
        .unwrap();
}

/// Width and height from a PNG's IHDR chunk.
fn png_size(path: &Path) -> (u32, u32) {
    let bytes = fs::read(path).unwrap();
    assert_eq!(&bytes[1..4], b"PNG", "{} is not a PNG", path.display());
    let read = |offset: usize| u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    return (read(16), read(20));
}

#[test]
fn renders_scene_to_png_with_camera_size() {
    let directory = directory("renders");
    let scene = write_scene(&directory, SCENE);
    let image = directory.join("out.png");
    let output = run(&[scene.as_os_str(), image.as_os_str()]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(png_size(&image), (24, 16));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Image saved at"));
}

#[test]
fn quiet_mode_prints_nothing() {
    let directory = directory("quiet");
    let scene = write_scene(&directory, SCENE);
    let image = directory.join("out.png");
    let output = run(&["-q".as_ref(), scene.as_os_str(), image.as_os_str()]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn serial_and_parallel_modes_write_identical_images() {
    let directory = directory("modes");
    let scene = write_scene(&directory, SCENE);
    let images = ["serial", "parallel"].map(|mode| {
        let image = directory.join(format!("{mode}.png"));
        let args = [
            "-q".as_ref(),
            "-r".as_ref(),
            mode.as_ref(),
            scene.as_os_str(),
            image.as_os_str(),
        ];
        assert!(run(&args).status.success());
        return fs::read(image).unwrap();
    });
    assert!(images[0] == images[1]);
}

#[test]
fn invalid_scene_reports_line_and_column() {
    let directory = directory("invalid");
    let scene = write_scene(&directory, &format!("{SCENE}  colr: [1, 0, 0]\n"));
    let output = run(&[scene.as_os_str(), directory.join("out.png").as_os_str()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown field `colr`"), "{stderr}");
    assert!(stderr.contains("line 16 column 3"), "{stderr}");
}

#[test]
fn missing_scene_file_reports_its_path() {
    let directory = directory("missing");
    let scene = directory.join("does-not-exist.yaml");
    let output = run(&[scene.as_os_str(), directory.join("out.png").as_os_str()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does-not-exist.yaml"), "{stderr}");
}

#[test]
fn image_textures_load_relative_to_the_scene_file() {
    let directory = directory("image_texture");
    fs::create_dir_all(directory.join("textures")).unwrap();
    image::RgbImage::from_pixel(1, 1, image::Rgb([255, 0, 0]))
        .save(directory.join("textures/red.png"))
        .unwrap();
    // a sphere filling the view, lit only by its ambient color: pixels show the texture as is
    let scene = write_scene(
        &directory,
        "
- add: camera
  width: 9
  height: 9
  field-of-view: 0.5
  from: [0, 0, -5]
  to: [0, 0, 0]
  up: [0, 1, 0]
- add: light
  at: [-10, 10, -10]
  intensity: [1, 1, 1]
- add: sphere
  material:
    ambient: 1
    diffuse: 0
    specular: 0
    pattern:
      type: map
      mapping: spherical
      uv_pattern:
        type: image
        file: textures/red.png
",
    );
    let image = directory.join("out.png");
    // run from another working directory: the path must be resolved from the scene file
    let output = Command::new(env!("CARGO_BIN_EXE_ray-tracer-cli"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([scene.as_os_str(), image.as_os_str()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let rendered = image::open(&image).unwrap().to_rgb8();
    assert_eq!(rendered.get_pixel(4, 4).0, [255, 0, 0]);
}

#[test]
fn missing_texture_image_reports_its_path() {
    let directory = directory("missing_texture");
    let scene = write_scene(
        &directory,
        &format!(
            "{SCENE}  material:\n    pattern:\n      type: map\n      mapping: spherical\n      uv_pattern:\n        type: image\n        file: missing.png\n"
        ),
    );
    let output = run(&[scene.as_os_str(), directory.join("out.png").as_os_str()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot load texture"), "{stderr}");
    assert!(stderr.contains("missing.png"), "{stderr}");
}
