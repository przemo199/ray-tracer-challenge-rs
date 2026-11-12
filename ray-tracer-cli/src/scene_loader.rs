//! Loads YAML scene files: `format` describes the file, `resolve` builds the world from it.

use core::fmt::{self, Debug, Display, Formatter};
use ray_tracer::composites::{Camera, World};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

mod format;
mod resolve;

pub fn load_scene_description<P: AsRef<Path>>(path: P) -> Result<(World, Camera), SceneError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| SceneError::Io {
        path: path.to_owned(),
        source,
    })?;
    // texture images are looked up relative to the scene file
    let base_dir = path.parent().unwrap_or(Path::new(""));
    return parse_scene(&text, base_dir);
}

/// Parses a scene, resolving relative paths in it (texture images) from `base_dir`.
pub fn parse_scene(text: &str, base_dir: &Path) -> Result<(World, Camera), SceneError> {
    let entries: Vec<format::Entry> = serde_saphyr::from_str(text).map_err(SceneError::Yaml)?;
    return resolve::build(&entries, base_dir);
}

pub enum SceneError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Malformed YAML or a value of the wrong shape; the message includes line and column.
    Yaml(serde_saphyr::Error),
    Undefined {
        kind: &'static str,
        name: String,
    },
    MissingCamera,
    Invalid(String),
    /// A texture image that is missing or can't be decoded (only PNG and JPEG are supported).
    Image {
        path: PathBuf,
        source: Box<dyn Error + Send + Sync>,
    },
}

impl Display for SceneError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        return match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Yaml(error) => write!(formatter, "invalid scene: {error}"),
            Self::Undefined { kind, name } => write!(formatter, "undefined {kind} `{name}`"),
            Self::MissingCamera => formatter.write_str("the scene has no camera"),
            Self::Invalid(message) => formatter.write_str(message),
            Self::Image { path, source } => {
                write!(
                    formatter,
                    "cannot load texture {}: {source}",
                    path.display()
                )
            }
        };
    }
}

// `main` reports errors with `Debug`, so make it as readable as `Display`.
impl Debug for SceneError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        return Display::fmt(self, formatter);
    }
}

impl Error for SceneError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            Self::Io { source, .. } => Some(source),
            Self::Yaml(error) => Some(error),
            Self::Image { source, .. } => Some(source.as_ref()),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ray_tracer::patterns::{
        ColorProvider, CubeMap, CylindricalMapping, Mapping, PlanarMapping, SphericalMapping,
        TextureMap, UvAlignCheck, UvCheckers, UvPattern,
    };
    use ray_tracer::primitives::{Color, Point, transformations};
    use ray_tracer::shapes::{Shape, Transform};

    const CAMERA: &str = "
- add: camera
  width: 10
  height: 10
  field-of-view: 1
  from: [0, 0, -5]
  to: [0, 0, 0]
  up: [0, 1, 0]
";

    fn parse(entries: &str) -> Result<(World, Camera), SceneError> {
        return parse_scene(&format!("{CAMERA}{entries}"), Path::new(""));
    }

    fn error(entries: &str) -> String {
        return parse(entries)
            .expect_err("scene should be rejected")
            .to_string();
    }

    fn only_shape(world: &World) -> &dyn Shape {
        assert_eq!(world.shapes.len(), 1);
        return world.shapes[0].as_ref();
    }

    #[test]
    fn every_example_scene_loads() {
        let scenes = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scenes");
        let mut count = 0;
        for entry in fs::read_dir(scenes).unwrap() {
            let path = entry.unwrap().path();
            let (world, _) = load_scene_description(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert!(!world.shapes.is_empty(), "{}", path.display());
            count += 1;
        }
        assert!(count >= 8);
    }

    #[test]
    fn numbers_may_be_written_as_integers_or_floats() {
        let (world, _) =
            parse("- add: light\n  at: [1, .5, 2.]\n  intensity: [1, 1.0, 0]").unwrap();
        assert_eq!(world.lights[0].position, Point::new(1, 0.5, 2));
        assert_eq!(world.lights[0].intensity, Color::new(1, 1, 0));
    }

    #[test]
    fn materials_extend_their_base_and_resolve_named_colors() {
        let (world, _) = parse(
            "
- define: red-color
  value: [1, 0, 0]
- define: base-material
  value: { color: red-color, diffuse: 0.2, specular: 0.3 }
- define: shiny-material
  extend: base-material
  value: { specular: 0.9 }
- add: sphere
  material: shiny-material
",
        )
        .unwrap();
        let material = only_shape(&world).material();
        assert_eq!(material.color, Color::new(1, 0, 0));
        assert_eq!(material.diffuse, 0.2);
        assert_eq!(material.specular, 0.9);
    }

    #[test]
    fn shape_shadow_flag_overrides_material() {
        let (world, _) = parse("- add: cube\n  shadow: false").unwrap();
        assert!(!only_shape(&world).material().casts_shadow);
    }

    #[test]
    fn transform_steps_apply_in_order_wherever_names_appear() {
        let named = parse(
            "
- define: move-x
  value: [[translate, 1, 0, 0]]
- add: sphere
  transform: [[scale, 2, 2, 2], move-x]
",
        )
        .unwrap();
        let inline = parse("- add: sphere\n  transform: [[scale, 2, 2, 2], [translate, 1, 0, 0]]");
        assert_eq!(only_shape(&named.0), only_shape(&inline.unwrap().0));
    }

    #[test]
    fn misspelled_key_is_rejected() {
        assert!(error("- add: sphere\n  materail: {}").contains("materail"));
        assert!(error("- add: sphere\n  material: { colr: [1, 0, 0] }").contains("colr"));
    }

    #[test]
    fn undefined_names_are_reported() {
        assert_eq!(
            error("- add: sphere\n  material: glass-material"),
            "undefined material `glass-material`"
        );
        assert_eq!(
            error("- add: sphere\n  transform: [big-object]"),
            "undefined transform `big-object`"
        );
    }

    #[test]
    fn malformed_transform_steps_are_rejected() {
        assert!(error("- add: sphere\n  transform: [[rotate-y, 1, 2]]").contains("length"));
        assert!(error("- add: sphere\n  transform: [[scale, 2]]").contains("length"));
        assert!(error("- add: sphere\n  transform: [[shrink, 2]]").contains("shrink"));
    }

    #[test]
    fn unknown_entries_and_misplaced_options_are_rejected() {
        assert!(error("- add: torus").contains("torus"));
        assert!(error("- remove: sphere").contains("remove"));
        assert!(error("- add: sphere\n  closed: true").contains("only apply to cylinders"));
    }

    #[test]
    fn scene_without_camera_is_rejected() {
        let result = parse_scene("- add: sphere", Path::new(""));
        assert_eq!(result.err().unwrap().to_string(), "the scene has no camera");
    }

    #[test]
    fn extend_chains_over_several_levels() {
        let (world, _) = parse(
            "
- define: base-material
  value: { ambient: 0.3, diffuse: 0.4, specular: 0.5 }
- define: middle-material
  extend: base-material
  value: { diffuse: 0.6 }
- define: top-material
  extend: middle-material
  value: { specular: 0.7 }
- add: sphere
  material: top-material
",
        )
        .unwrap();
        let material = only_shape(&world).material();
        assert_eq!(
            (material.ambient, material.diffuse, material.specular),
            (0.3, 0.6, 0.7)
        );
    }

    #[test]
    fn pattern_transform_is_applied() {
        let (world, _) = parse(
            "
- add: sphere
  material:
    pattern:
      type: stripes
      colors: [[1, 0, 0], [0, 0, 1]]
      transform: [[scale, 0.5, 1, 1]]
",
        )
        .unwrap();
        let shape = only_shape(&world);
        let pattern = shape.material().pattern.as_ref().unwrap();
        // x = 0.75 lands in the second stripe once the stripes are scaled to half width
        let color = pattern.color_at_shape(shape, &Point::new(0.75, 0, 0));
        assert_eq!(color, Color::new(0, 0, 1));
    }

    #[test]
    fn non_invertible_transforms_are_rejected() {
        assert!(error("- add: sphere\n  transform: [[scale, 0, 1, 1]]").contains("not invertible"));
        let define =
            "- define: flat\n  value: [[scale, 1, 0, 1]]\n- add: cube\n  transform: [flat]";
        assert!(error(define).contains("not invertible"));
    }

    #[test]
    fn camera_looking_at_its_own_position_is_rejected() {
        let scene = CAMERA.replace("to: [0, 0, 0]", "to: [0, 0, -5]");
        let message = parse_scene(&scene, Path::new(""))
            .expect_err("should be rejected")
            .to_string();
        assert!(message.starts_with("camera:"), "{message}");
    }

    #[test]
    fn only_materials_can_extend() {
        let message =
            error("- define: big-transform\n  extend: other\n  value: [[scale, 2, 2, 2]]");
        assert_eq!(message, "`big-transform`: only materials can use `extend`");
    }

    fn only_pattern(world: &World) -> &dyn ColorProvider {
        let pattern = only_shape(world).material().pattern.as_ref();
        return pattern.expect("the shape should have a pattern").as_ref();
    }

    #[rstest::rstest]
    #[case("planar", Box::new(PlanarMapping))]
    #[case("spherical", Box::new(SphericalMapping))]
    #[case("cylindrical", Box::new(CylindricalMapping))]
    fn texture_maps_load_with_each_mapping(#[case] name: &str, #[case] mapping: Box<dyn Mapping>) {
        let (world, _) = parse(&format!(
            "
- add: sphere
  material:
    pattern:
      type: map
      mapping: {name}
      uv_pattern:
        type: checkers
        width: 16
        height: 8
        colors: [[0, 0.5, 0], [1, 1, 1]]
"
        ))
        .unwrap();
        let checkers = UvCheckers::new(16, 8, Color::new(0, 0.5, 0), Color::WHITE);
        let expected = TextureMap::new(Box::new(checkers), mapping);
        assert!(only_pattern(&world) == &expected as &dyn ColorProvider);
    }

    #[test]
    fn align_check_loads_with_named_colors_and_transform() {
        let (world, _) = parse(
            "
- define: red-color
  value: [1, 0, 0]
- add: cylinder
  material:
    pattern:
      type: map
      mapping: cylindrical
      transform: [[rotate-y, 1]]
      uv_pattern:
        type: align_check
        colors:
          main: [1, 1, 1]
          ul: red-color
          ur: [1, 1, 0]
          bl: [0, 1, 0]
          br: [0, 1, 1]
",
        )
        .unwrap();
        let align_check = UvAlignCheck::new(
            Color::WHITE,
            Color::RED,
            Color::new(1, 1, 0),
            Color::GREEN,
            Color::new(0, 1, 1),
        );
        let mut expected = TextureMap::new(Box::new(align_check), Box::new(CylindricalMapping));
        expected.set_transformation(transformations::rotation_y(1));
        assert!(only_pattern(&world) == &expected as &dyn ColorProvider);
    }

    #[test]
    fn invalid_texture_maps_are_rejected() {
        let map = |body: &str| {
            return error(&format!(
                "- add: sphere\n  material:\n    pattern:\n      type: map\n{body}"
            ));
        };
        let checkers = "        type: checkers\n        width: 2\n        height: 2\n        colors: [[0, 0, 0], [1, 1, 1]]\n";
        let message = map(&format!(
            "      mapping: cubic\n      uv_pattern:\n{checkers}"
        ));
        assert!(message.contains("cubic"), "{message}");
        let message = map("      mapping: planar\n      uv_pattern:\n        type: noise\n");
        assert!(message.contains("noise"), "{message}");
        let message = map(&format!(
            "      mapping: planar\n      uv_pattern:\n{}",
            checkers.replace("width", "widht")
        ));
        assert!(message.contains("widht"), "{message}");
        let message = map(&format!(
            "      mapping: planar\n      colors: [[0, 0, 0], [1, 1, 1]]\n      uv_pattern:\n{checkers}"
        ));
        assert!(message.contains("colors"), "{message}");
    }

    /// A texture-map pattern on a cube, with `body` as its keys after `type: map`.
    fn cube_with_map(body: &str) -> Result<(World, Camera), SceneError> {
        return parse(&format!(
            "- add: cube\n  material:\n    pattern:\n      type: map\n{body}"
        ));
    }

    fn face(main: &str) -> String {
        return format!(
            "        type: align_check\n        colors: {{ main: {main}, ul: [1, 0, 0], ur: [1, 1, 0], bl: [0, 1, 0], br: [0, 1, 1] }}\n"
        );
    }

    #[test]
    fn cube_maps_load_one_pattern_per_face() {
        let faces = ["left", "front", "right", "back", "up", "down"];
        let mut body = "      mapping: cube\n".to_owned();
        for (index, name) in faces.iter().enumerate() {
            body += &format!("      {name}:\n{}", face(&format!("[0.{index}, 0, 0]")));
        }
        let (world, _) = cube_with_map(&body).unwrap();
        let face = |index: u8| -> Box<dyn UvPattern> {
            return Box::new(UvAlignCheck::new(
                Color::new(f64::from(index) / 10.0, 0, 0),
                Color::RED,
                Color::new(1, 1, 0),
                Color::GREEN,
                Color::new(0, 1, 1),
            ));
        };
        let expected = CubeMap::new(face(0), face(1), face(2), face(3), face(4), face(5));
        assert!(only_pattern(&world) == &expected as &dyn ColorProvider);
    }

    #[test]
    fn invalid_cube_maps_are_rejected() {
        let five_faces: String = ["left", "front", "right", "back", "up"]
            .iter()
            .map(|name| format!("      {name}:\n{}", face("[1, 1, 1]")))
            .collect();
        let message = error_of(cube_with_map(&format!("      mapping: cube\n{five_faces}")));
        assert_eq!(message, "cube map: missing the `down` face");
        let message = error_of(cube_with_map(&format!(
            "      mapping: cube\n      uv_pattern:\n{}",
            face("[1, 1, 1]")
        )));
        assert!(message.contains("instead of `uv_pattern`"), "{message}");
        let message = error_of(cube_with_map(&format!(
            "      mapping: spherical\n      front:\n{}",
            face("[1, 1, 1]")
        )));
        assert!(
            message.contains("only apply to `mapping: cube`"),
            "{message}"
        );
        let message = error_of(cube_with_map("      mapping: planar\n"));
        assert_eq!(message, "a texture map needs a `uv_pattern`");
    }

    fn error_of(result: Result<(World, Camera), SceneError>) -> String {
        return result.expect_err("scene should be rejected").to_string();
    }

    #[test]
    fn texture_images_are_resolved_relative_to_the_scene_file() {
        let scene = format!(
            "{CAMERA}
- add: sphere
  material:
    pattern:
      type: map
      mapping: spherical
      uv_pattern:
        type: image
        file: textures/earth.png
"
        );
        let message = parse_scene(&scene, Path::new("scenes"))
            .expect_err("the image doesn't exist")
            .to_string();
        let expected_path = Path::new("scenes").join("textures/earth.png");
        let expected = format!("cannot load texture {}", expected_path.display());
        assert!(message.starts_with(&expected), "{message}");
    }
}
