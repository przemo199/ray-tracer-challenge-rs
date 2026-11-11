//! Turns the parsed scene entries into a [`World`] and [`Camera`].

use super::SceneError;
use super::format::{
    CameraDef, ColorPatternDef, Define, DefineValue, Entry, LightDef, MappingKind, MaterialDef,
    PatternDef, Ref, Rgb, ShapeDef, ShapeKind, Step, TextureMapDef, UvPatternDef,
};
use ray_tracer::composites::{Camera, Material, World};
use ray_tracer::consts::{MAX, MIN};
use ray_tracer::patterns::{
    CheckerPattern, ColorProvider, CubeMap, CylindricalMapping, GradientPattern, Mapping,
    PlanarMapping, RingPattern, SphericalMapping, StripePattern, TextureMap, UvAlignCheck,
    UvCheckers, UvImage, UvPattern,
};
use ray_tracer::primitives::{Color, Light, Point, Transformation, Vector, transformations};
use ray_tracer::shapes::{Cone, Cube, Cylinder, Plane, Shape, Sphere, Transform};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// `base_dir` is where relative paths in the scene (texture images) are resolved from.
pub fn build(entries: Vec<Entry>, base_dir: &Path) -> Result<(World, Camera), SceneError> {
    // Definitions first, so `add` entries may refer to names defined further down the file.
    let mut definitions = Definitions {
        base_dir: base_dir.to_owned(),
        ..Definitions::default()
    };
    for entry in &entries {
        if let Entry::Define(name, define) = entry {
            definitions.add(name, define)?;
        }
    }

    let mut world = World::new(Vec::new(), Vec::new());
    let mut camera = None;
    for entry in entries {
        match entry {
            Entry::Define(..) => {}
            Entry::Camera(definition) => camera = Some(build_camera(&definition)?),
            Entry::Light(LightDef { at, intensity }) => {
                world
                    .lights
                    .push(Light::new(Point::from(at), Color::from(intensity)));
            }
            Entry::Shape(kind, definition) => {
                world.shapes.push(definitions.shape(kind, &definition)?);
            }
        }
    }
    let camera = camera.ok_or(SceneError::MissingCamera)?;
    return Ok((world, camera));
}

fn build_camera(definition: &CameraDef) -> Result<Camera, SceneError> {
    let mut camera = Camera::new(
        definition.width,
        definition.height,
        definition.field_of_view,
    );
    let view = transformations::view_transform(
        Point::from(definition.from),
        Point::from(definition.to),
        Vector::from(definition.up),
    );
    if !view.is_invertible() {
        return Err(SceneError::Invalid(
            "camera: `from` must differ from `to`, and `up` must not point along the view direction"
                .to_owned(),
        ));
    }
    camera.set_transformation(view);
    return Ok(camera);
}

#[derive(Default)]
struct Definitions {
    base_dir: PathBuf,
    colors: HashMap<String, Color>,
    /// Stored with `extend` already applied.
    materials: HashMap<String, MaterialDef>,
    transformations: HashMap<String, Transformation>,
}

impl Definitions {
    fn add(&mut self, name: &str, define: &Define) -> Result<(), SceneError> {
        match &define.value {
            DefineValue::Material(material) => {
                let mut material = material.clone();
                material.extend = material.extend.or_else(|| define.extend.clone());
                let material = self.flatten(material)?;
                self.materials.insert(name.to_owned(), material);
            }
            _ if define.extend.is_some() => {
                return Err(SceneError::Invalid(format!(
                    "`{name}`: only materials can use `extend`"
                )));
            }
            DefineValue::Transform(steps) => {
                let transformation = self.transformation(steps)?;
                self.transformations.insert(name.to_owned(), transformation);
            }
            DefineValue::Color(rgb) => {
                self.colors.insert(name.to_owned(), Color::from(*rgb));
            }
        }
        return Ok(());
    }

    /// Layers a material over the definition it extends, if any.
    fn flatten(&self, mut material: MaterialDef) -> Result<MaterialDef, SceneError> {
        let Some(base_name) = material.extend.take() else {
            return Ok(material);
        };
        let base = lookup(&self.materials, "material", &base_name)?.clone();
        return Ok(MaterialDef {
            extend: None,
            color: material.color.or(base.color),
            pattern: material.pattern.or(base.pattern),
            ambient: material.ambient.or(base.ambient),
            diffuse: material.diffuse.or(base.diffuse),
            specular: material.specular.or(base.specular),
            shininess: material.shininess.or(base.shininess),
            reflective: material.reflective.or(base.reflective),
            transparency: material.transparency.or(base.transparency),
            refractive_index: material.refractive_index.or(base.refractive_index),
            casts_shadow: material.casts_shadow.or(base.casts_shadow),
        });
    }

    fn material(&self, reference: &Ref<MaterialDef>) -> Result<Material, SceneError> {
        let definition = match reference {
            Ref::Named(name) => lookup(&self.materials, "material", name)?.clone(),
            Ref::Inline(definition) => self.flatten(definition.clone())?,
        };
        let mut material = Material::default();
        if let Some(color) = &definition.color {
            material.color = self.color(color)?;
        }
        if let Some(pattern) = &definition.pattern {
            material.pattern = Some(self.pattern(pattern)?);
        }
        let set = |field: &mut f64, value: Option<f64>| *field = value.unwrap_or(*field);
        set(&mut material.ambient, definition.ambient);
        set(&mut material.diffuse, definition.diffuse);
        set(&mut material.specular, definition.specular);
        set(&mut material.shininess, definition.shininess);
        set(&mut material.reflectiveness, definition.reflective);
        set(&mut material.transparency, definition.transparency);
        set(&mut material.refractive_index, definition.refractive_index);
        material.casts_shadow = definition.casts_shadow.unwrap_or(material.casts_shadow);
        return Ok(material);
    }

    fn color(&self, reference: &Ref<Rgb>) -> Result<Color, SceneError> {
        return match reference {
            Ref::Named(name) => lookup(&self.colors, "color", name).copied(),
            Ref::Inline(rgb) => Ok(Color::from(*rgb)),
        };
    }

    fn pattern(&self, definition: &PatternDef) -> Result<Box<dyn ColorProvider>, SceneError> {
        return match definition {
            PatternDef::Stripes(def) => self.color_pattern(def, StripePattern::new),
            PatternDef::Gradient(def) => self.color_pattern(def, GradientPattern::new),
            PatternDef::Rings(def) => self.color_pattern(def, RingPattern::new),
            PatternDef::Checkers(def) => self.color_pattern(def, CheckerPattern::new),
            PatternDef::Map(def) => self.texture_map(def),
        };
    }

    /// Builds a two-color pattern with its constructor, `new(color_a, color_b)`.
    fn color_pattern<P: Transform + ColorProvider>(
        &self,
        definition: &ColorPatternDef,
        new: fn(Color, Color) -> P,
    ) -> Result<Box<dyn ColorProvider>, SceneError> {
        let [color_a, color_b] = &definition.colors;
        let pattern = new(self.color(color_a)?, self.color(color_b)?);
        return self.transformed(pattern, definition.transform.as_deref());
    }

    fn texture_map(
        &self,
        definition: &TextureMapDef,
    ) -> Result<Box<dyn ColorProvider>, SceneError> {
        let mapping: Box<dyn Mapping> = match definition.mapping {
            MappingKind::Planar => Box::new(PlanarMapping),
            MappingKind::Spherical => Box::new(SphericalMapping),
            MappingKind::Cylindrical => Box::new(CylindricalMapping),
            MappingKind::Cube => return self.cube_map(definition),
        };
        let faces = [
            &definition.left,
            &definition.front,
            &definition.right,
            &definition.back,
            &definition.up,
            &definition.down,
        ];
        if faces.iter().any(|face| face.is_some()) {
            return Err(SceneError::Invalid(
                "`left`, `front`, `right`, `back`, `up` and `down` only apply to `mapping: cube`"
                    .to_owned(),
            ));
        }
        let uv_pattern = definition.uv_pattern.as_ref().ok_or_else(|| {
            return SceneError::Invalid("a texture map needs a `uv_pattern`".to_owned());
        })?;
        let texture = TextureMap::new(self.uv_pattern(uv_pattern)?, mapping);
        return self.transformed(texture, definition.transform.as_deref());
    }

    fn cube_map(&self, definition: &TextureMapDef) -> Result<Box<dyn ColorProvider>, SceneError> {
        if definition.uv_pattern.is_some() {
            return Err(SceneError::Invalid(
                "a cube map takes one pattern per face (`left`, `front`, `right`, `back`, `up`, \
                 `down`) instead of `uv_pattern`"
                    .to_owned(),
            ));
        }
        let face = |name: &str, face: &Option<UvPatternDef>| {
            let face = face.as_ref().ok_or_else(|| {
                return SceneError::Invalid(format!("cube map: missing the `{name}` face"));
            })?;
            return self.uv_pattern(face);
        };
        let cube_map = CubeMap::new(
            face("left", &definition.left)?,
            face("front", &definition.front)?,
            face("right", &definition.right)?,
            face("back", &definition.back)?,
            face("up", &definition.up)?,
            face("down", &definition.down)?,
        );
        return self.transformed(cube_map, definition.transform.as_deref());
    }

    /// Applies a pattern's optional `transform` and boxes it.
    fn transformed<P: Transform + ColorProvider>(
        &self,
        mut pattern: P,
        steps: Option<&[Step]>,
    ) -> Result<Box<dyn ColorProvider>, SceneError> {
        if let Some(steps) = steps {
            pattern.set_transformation(self.transformation(steps)?);
        }
        return Ok(Box::new(pattern));
    }

    fn uv_pattern(&self, definition: &UvPatternDef) -> Result<Box<dyn UvPattern>, SceneError> {
        return Ok(match definition {
            UvPatternDef::Checkers {
                width,
                height,
                colors: [a, b],
            } => Box::new(UvCheckers::new(
                *width,
                *height,
                self.color(a)?,
                self.color(b)?,
            )),
            UvPatternDef::AlignCheck { colors } => Box::new(UvAlignCheck::new(
                self.color(&colors.main)?,
                self.color(&colors.ul)?,
                self.color(&colors.ur)?,
                self.color(&colors.bl)?,
                self.color(&colors.br)?,
            )),
            UvPatternDef::Image { file } => {
                let path = self.base_dir.join(file);
                let image = UvImage::from_path(&path).map_err(|source| SceneError::Image {
                    path,
                    source: Box::new(source),
                })?;
                Box::new(image)
            }
        });
    }

    /// Steps apply in list order: `[a, b]` means "a, then b", i.e. the matrix `b * a`.
    fn transformation(&self, steps: &[Step]) -> Result<Transformation, SceneError> {
        let mut transformation = Transformation::IDENTITY;
        for step in steps {
            let matrix = match step {
                Ref::Named(name) => *lookup(&self.transformations, "transform", name)?,
                Ref::Inline(op) => op.0,
            };
            transformation = matrix * transformation;
        }
        // shapes and patterns store the inverse, which would otherwise panic
        if !transformation.is_invertible() {
            return Err(SceneError::Invalid(
                "transform is not invertible (is something scaled by 0?)".to_owned(),
            ));
        }
        return Ok(transformation);
    }

    fn shape(&self, kind: ShapeKind, definition: &ShapeDef) -> Result<Box<dyn Shape>, SceneError> {
        let mut material = match &definition.material {
            Some(reference) => self.material(reference)?,
            None => Material::default(),
        };
        if let Some(shadow) = definition.shadow {
            material.casts_shadow = shadow;
        }
        let transformation = self.transformation(&definition.transform)?;
        let (min, max, closed) = (definition.min, definition.max, definition.closed);
        let is_capped = matches!(kind, ShapeKind::Cylinder | ShapeKind::Cone);
        if !is_capped && (min.is_some() || max.is_some() || closed.is_some()) {
            return Err(SceneError::Invalid(format!(
                "{kind:?}: `min`, `max` and `closed` only apply to cylinders and cones"
            )));
        }
        let (min, max, closed) = (
            min.unwrap_or(MIN),
            max.unwrap_or(MAX),
            closed.unwrap_or(false),
        );
        return Ok(match kind {
            ShapeKind::Plane => Box::new(Plane::new(material, transformation)),
            ShapeKind::Sphere => Box::new(Sphere::new(material, transformation)),
            ShapeKind::Cube => Box::new(Cube::new(material, transformation)),
            ShapeKind::Cylinder => {
                Box::new(Cylinder::new(material, transformation, min, max, closed))
            }
            ShapeKind::Cone => Box::new(Cone::new(material, transformation, min, max, closed)),
        });
    }
}

fn lookup<'a, T>(
    map: &'a HashMap<String, T>,
    kind: &'static str,
    name: &str,
) -> Result<&'a T, SceneError> {
    return map.get(name).ok_or_else(|| SceneError::Undefined {
        kind,
        name: name.to_owned(),
    });
}
