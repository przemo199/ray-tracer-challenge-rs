//! Typed model of the scene YAML format (the one used in *The Ray Tracer Challenge*).
//!
//! A scene is a list of entries. `add:` places a camera, light or shape; `define:` names a
//! material, transform or color that later entries refer to by name. Every mapping rejects unknown
//! keys, so typos are reported instead of being ignored.

use core::fmt::{self, Formatter};
use core::marker::PhantomData;
use ray_tracer::primitives::{Transformation, transformations};
use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
use serde::de::{Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

pub type Rgb = [f64; 3];

#[allow(
    clippy::large_enum_variant,
    reason = "parsed once per scene; boxing adds noise"
)]
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Define(String, Define),
    Camera(CameraDef),
    Light(LightDef),
    Shape(ShapeKind, ShapeDef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    Plane,
    Sphere,
    Cube,
    Cylinder,
    Cone,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CameraDef {
    pub width: u32,
    pub height: u32,
    pub field_of_view: f64,
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub up: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightDef {
    pub at: [f64; 3],
    pub intensity: Rgb,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeDef {
    pub material: Option<Ref<MaterialDef>>,
    #[serde(default)]
    pub transform: Vec<Step>,
    /// `false` stops the shape from casting shadows (overrides the material's `casts-shadow`).
    pub shadow: Option<bool>,
    /// Cylinders and cones only.
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub closed: Option<bool>,
}

/// Every field is optional so that `extend` can layer a definition over its base.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MaterialDef {
    pub extend: Option<String>,
    pub color: Option<Ref<Rgb>>,
    pub pattern: Option<PatternDef>,
    pub ambient: Option<f64>,
    pub diffuse: Option<f64>,
    pub specular: Option<f64>,
    pub shininess: Option<f64>,
    pub reflective: Option<f64>,
    pub transparency: Option<f64>,
    pub refractive_index: Option<f64>,
    pub casts_shadow: Option<bool>,
}

/// A material's pattern, selected by its `type:` key.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatternDef {
    Stripes(ColorPatternDef),
    Gradient(ColorPatternDef),
    Rings(ColorPatternDef),
    Checkers(ColorPatternDef),
    /// A texture map: a UV pattern painted on through a mapping. Boxed: it is far larger than the
    /// other variants, and materials (which hold patterns) are cloned for `extend`.
    Map(Box<TextureMapDef>),
}

/// The 3D patterns, which alternate or blend two colors.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorPatternDef {
    pub colors: [Ref<Rgb>; 2],
    pub transform: Option<Vec<Step>>,
}

/// `uv_pattern` for the planar/spherical/cylindrical mappings; one pattern per face (`left`, `front`,
/// `right`, `back`, `up`, `down`) for `mapping: cube`. `resolve.rs` checks the combination.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureMapDef {
    pub mapping: MappingKind,
    pub uv_pattern: Option<UvPatternDef>,
    pub left: Option<UvPatternDef>,
    pub front: Option<UvPatternDef>,
    pub right: Option<UvPatternDef>,
    pub back: Option<UvPatternDef>,
    pub up: Option<UvPatternDef>,
    pub down: Option<UvPatternDef>,
    pub transform: Option<Vec<Step>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingKind {
    Planar,
    Spherical,
    Cylindrical,
    Cube,
}

/// A 2D pattern for a texture map, selected by its `type:` key.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum UvPatternDef {
    Checkers {
        width: u32,
        height: u32,
        colors: [Ref<Rgb>; 2],
    },
    AlignCheck {
        colors: AlignCheckColors,
    },
    /// A PNG or JPEG file; a relative path is relative to the scene file.
    Image {
        file: PathBuf,
    },
}

/// The main color and the four corner colors (upper/bottom, left/right) of an align check.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlignCheckColors {
    pub main: Ref<Rgb>,
    pub ul: Ref<Rgb>,
    pub ur: Ref<Rgb>,
    pub bl: Ref<Rgb>,
    pub br: Ref<Rgb>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Define {
    pub extend: Option<String>,
    pub value: DefineValue,
}

#[allow(
    clippy::large_enum_variant,
    reason = "parsed once per scene; boxing adds noise"
)]
#[derive(Clone, Debug, PartialEq)]
pub enum DefineValue {
    Material(MaterialDef),
    Transform(Vec<Step>),
    Color(Rgb),
}

/// A transform step: the name of a defined transform, or an operation such as `[scale, 1, 2, 3]`.
pub type Step = Ref<Op>;

/// A single transformation, already turned into its matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Op(pub Transformation);

/// A value written inline, or the name of a `define`d one.
#[derive(Clone, Debug, PartialEq)]
pub enum Ref<T> {
    Named(String),
    Inline(T),
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter.write_str("a mapping starting with `add` or `define`");
            }

            // The first key says what the entry is; the remaining keys are read into that type.
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Entry, A::Error> {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| A::Error::custom("empty scene entry"))?;
                let name: String = map.next_value()?;
                let rest = MapAccessDeserializer::new(map);
                let kind = match (key.as_str(), name.as_str()) {
                    ("define", _) => return Ok(Entry::Define(name, Define::deserialize(rest)?)),
                    ("add", "camera") => return Ok(Entry::Camera(CameraDef::deserialize(rest)?)),
                    ("add", "light") => return Ok(Entry::Light(LightDef::deserialize(rest)?)),
                    ("add", "plane") => ShapeKind::Plane,
                    ("add", "sphere") => ShapeKind::Sphere,
                    ("add", "cube") => ShapeKind::Cube,
                    ("add", "cylinder") => ShapeKind::Cylinder,
                    ("add", "cone") => ShapeKind::Cone,
                    ("add", other) => {
                        return Err(A::Error::unknown_variant(
                            other,
                            &[
                                "camera", "light", "plane", "sphere", "cube", "cylinder", "cone",
                            ],
                        ));
                    }
                    (other, _) => return Err(A::Error::unknown_field(other, &["add", "define"])),
                };
                return Ok(Entry::Shape(kind, ShapeDef::deserialize(rest)?));
            }
        }

        return deserializer.deserialize_map(EntryVisitor);
    }
}

impl<'de> Deserialize<'de> for DefineValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DefineValueVisitor;

        impl<'de> Visitor<'de> for DefineValueVisitor {
            type Value = DefineValue;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter
                    .write_str("a material mapping, a list of transforms, or a color [r, g, b]");
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<DefineValue, A::Error> {
                let material = MaterialDef::deserialize(MapAccessDeserializer::new(map))?;
                return Ok(DefineValue::Material(material));
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<DefineValue, A::Error> {
                let mut numbers = Vec::new();
                let mut steps = Vec::new();
                while let Some(item) = seq.next_element::<NumberOrStep>()? {
                    match item {
                        NumberOrStep::Number(number) => numbers.push(number),
                        NumberOrStep::Step(step) => steps.push(step),
                    }
                }
                return match (numbers.as_slice(), steps.is_empty()) {
                    ([], _) => Ok(DefineValue::Transform(steps)),
                    (&[red, green, blue], true) => Ok(DefineValue::Color([red, green, blue])),
                    _ => Err(A::Error::invalid_value(
                        serde::de::Unexpected::Seq,
                        &DefineValueVisitor,
                    )),
                };
            }
        }

        return deserializer.deserialize_any(DefineValueVisitor);
    }
}

/// Element of a `define`d list, which is either a color (numbers) or a transform (steps).
#[derive(Clone, Debug, PartialEq)]
enum NumberOrStep {
    Number(f64),
    Step(Step),
}

impl<'de> Deserialize<'de> for NumberOrStep {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ItemVisitor;

        impl<'de> Visitor<'de> for ItemVisitor {
            type Value = NumberOrStep;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter.write_str("a number, a transform name or a transform operation");
            }

            fn visit_f64<E: Error>(self, value: f64) -> Result<NumberOrStep, E> {
                return Ok(NumberOrStep::Number(value));
            }

            fn visit_i64<E: Error>(self, value: i64) -> Result<NumberOrStep, E> {
                return Ok(NumberOrStep::Number(value as f64));
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<NumberOrStep, E> {
                return Ok(NumberOrStep::Number(value as f64));
            }

            fn visit_str<E: Error>(self, name: &str) -> Result<NumberOrStep, E> {
                return Ok(NumberOrStep::Step(Ref::Named(name.to_owned())));
            }

            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<NumberOrStep, A::Error> {
                let op = Op::deserialize(SeqAccessDeserializer::new(seq))?;
                return Ok(NumberOrStep::Step(Ref::Inline(op)));
            }
        }

        return deserializer.deserialize_any(ItemVisitor);
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ref<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RefVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for RefVisitor<T> {
            type Value = Ref<T>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter.write_str("a defined name or an inline value");
            }

            fn visit_str<E: Error>(self, name: &str) -> Result<Ref<T>, E> {
                return Ok(Ref::Named(name.to_owned()));
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Ref<T>, A::Error> {
                return T::deserialize(MapAccessDeserializer::new(map)).map(Ref::Inline);
            }

            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Ref<T>, A::Error> {
                return T::deserialize(SeqAccessDeserializer::new(seq)).map(Ref::Inline);
            }
        }

        return deserializer.deserialize_any(RefVisitor(PhantomData));
    }
}

impl<'de> Deserialize<'de> for Op {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OpVisitor;

        impl<'de> Visitor<'de> for OpVisitor {
            type Value = Op;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                return formatter.write_str(
                    "[scale|translate, x, y, z] or [rotate-x|rotate-y|rotate-z, radians]",
                );
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Op, A::Error> {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::invalid_length(0, &self))?;
                let arity = match name.as_str() {
                    "scale" | "translate" => 3,
                    "rotate-x" | "rotate-y" | "rotate-z" => 1,
                    other => {
                        const OPS: &[&str] =
                            &["scale", "translate", "rotate-x", "rotate-y", "rotate-z"];
                        return Err(A::Error::unknown_variant(other, OPS));
                    }
                };
                let mut args = [0.0; 3];
                for (index, arg) in args.iter_mut().take(arity).enumerate() {
                    *arg = seq
                        .next_element()?
                        .ok_or_else(|| A::Error::invalid_length(index + 1, &self))?;
                }
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    return Err(A::Error::invalid_length(arity + 2, &self));
                }
                let [x, y, z] = args;
                let transformation = match name.as_str() {
                    "scale" => transformations::scaling(x, y, z),
                    "translate" => transformations::translation(x, y, z),
                    "rotate-x" => transformations::rotation_x(x),
                    "rotate-y" => transformations::rotation_y(x),
                    _ => transformations::rotation_z(x),
                };
                return Ok(Op(transformation));
            }
        }

        return deserializer.deserialize_seq(OpVisitor);
    }
}
