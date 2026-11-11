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
    return parse_scene(&text);
}

pub fn parse_scene(text: &str) -> Result<(World, Camera), SceneError> {
    let entries: Vec<format::Entry> = serde_saphyr::from_str(text).map_err(SceneError::Yaml)?;
    return resolve::build(entries);
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
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ray_tracer::primitives::{Color, Point};
    use ray_tracer::shapes::Shape;

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
        return parse_scene(&format!("{CAMERA}{entries}"));
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
        let result = parse_scene("- add: sphere");
        assert_eq!(result.err().unwrap().to_string(), "the scene has no camera");
    }
}
