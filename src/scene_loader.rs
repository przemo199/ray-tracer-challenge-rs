use crate::composites::{Camera, Material, World};
use crate::patterns::{CheckerPattern, GradientPattern, Pattern, RingPattern, StripePattern};
use crate::primitives::{transformations, Transformation};
use crate::primitives::{Color, Light, Point, Vector};
use crate::shapes::{Cone, Cube, Cylinder, Plane, Sphere};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use yaml_rust::Yaml::BadValue;
use yaml_rust::{Yaml, YamlLoader};

const ADD: &str = "add";
const COLOR: &str = "color";
const VALUE: &str = "value";
const TYPE: &str = "type";
const DEFINE: &str = "define";
const EXTEND: &str = "extend";
const TRANSFORMATION: &str = "transform";

#[derive(Debug)]
struct Definitions {
    colors: HashMap<String, Color>,
    materials: HashMap<String, Material>,
    patterns: HashMap<String, Arc<dyn Pattern>>,
    transformations: HashMap<String, Transformation>,
}

impl Definitions {
    pub fn new() -> Self {
        return Definitions {
            colors: HashMap::<String, Color>::new(),
            materials: HashMap::<String, Material>::new(),
            patterns: HashMap::<String, Arc<dyn Pattern>>::new(),
            transformations: HashMap::<String, Transformation>::new(),
        };
    }
}

fn load_file_to_yaml<P: AsRef<Path>>(path: P) -> Yaml {
    let file = fs::read_to_string(path).unwrap();
    let mut docs = YamlLoader::load_from_str(&file).unwrap();
    if docs.len() != 1 {
        panic!("Incorrect yaml format");
    }
    return docs.remove(0);
}

pub fn load_scene_description<P: AsRef<Path>>(path: P) -> (World, Camera) {
    let yaml = load_file_to_yaml(path);
    let mut definitions = Definitions::new();

    process_definitions(&yaml, &mut definitions);
    return parse_scene(&yaml, &definitions);
}

fn process_definitions(yaml: &Yaml, definitions: &mut Definitions) {
    for entry in yaml.clone().into_iter() {
        if let Yaml::String(name) = &entry[DEFINE] {
            if name.ends_with("-color") {
                let color = parse_color(&entry, definitions);
                definitions.colors.insert(name.clone(), color);
                continue;
            } else if name.ends_with("-material") {
                let material = parse_material_definition(&entry, definitions);
                definitions.materials.insert(name.clone(), material);
                continue;
            } else if name.ends_with("-transform") || name.ends_with("-object") {
                let transformation = parse_transformation_definition(&entry, definitions);
                definitions
                    .transformations
                    .insert(name.clone(), transformation);
            }
        }
    }
}

fn parse_f64(yaml: &Yaml) -> f64 {
    match yaml {
        Yaml::Integer(value) => {
            return *value as f64;
        }
        Yaml::Real(value) => {
            return value.parse().unwrap();
        }
        _ => {
            panic!("Incorrect float value");
        }
    }
}

fn parse_array_of_3(slice: &[Yaml]) -> [f64; 3] {
    let values: Vec<f64> = slice.iter().take(3).map(parse_f64).collect();
    return [values[0], values[1], values[2]];
}

fn parse_color(yaml: &Yaml, definitions: &Definitions) -> Color {
    match yaml {
        Yaml::Hash(_) => {
            if yaml[COLOR] != BadValue {
                return parse_color(&yaml[COLOR], definitions);
            } else {
                return parse_color(&yaml[VALUE], definitions);
            }
        }
        Yaml::Array(color_values) => {
            let color_channels = parse_array_of_3(color_values);
            return Color::new(color_channels[0], color_channels[1], color_channels[2]);
        }
        Yaml::String(key) => {
            return definitions.colors[key];
        }
        _ => {
            panic!("Incorrect color value");
        }
    }
}

fn parse_material_definition(yaml: &Yaml, definitions: &Definitions) -> Material {
    let mut material;
    if yaml[EXTEND] != BadValue {
        material = definitions.materials[yaml["extend"].as_str().unwrap()].clone();
    } else {
        material = Material::default();
    }

    let yaml = &yaml[VALUE];
    if yaml[COLOR] != BadValue {
        material.color = parse_color(&yaml[COLOR], definitions);
    }

    if yaml["pattern"] != BadValue {
        material.pattern = Some(parse_pattern(&yaml["pattern"], definitions));
    }

    if yaml["ambient"] != BadValue {
        material.ambient = parse_f64(&yaml["ambient"]);
    }

    if yaml["diffuse"] != BadValue {
        material.diffuse = parse_f64(&yaml["diffuse"]);
    }

    if yaml["specular"] != BadValue {
        material.specular = parse_f64(&yaml["specular"]);
    }

    if yaml["shininess"] != BadValue {
        material.shininess = parse_f64(&yaml["shininess"]);
    }

    if yaml["reflective"] != BadValue {
        material.reflectiveness = parse_f64(&yaml["reflective"]);
    }

    if yaml["transparency"] != BadValue {
        material.transparency = parse_f64(&yaml["transparency"]);
    }

    if yaml["refractive-index"] != BadValue {
        material.refractive_index = parse_f64(&yaml["refractive-index"]);
    }

    return material;
}

fn parse_material(yaml: &Yaml, definitions: &Definitions) -> Material {
    if let Yaml::String(name) = yaml {
        return definitions.materials[name].clone();
    }
    let mut material;
    if yaml[EXTEND] != BadValue {
        material = definitions.materials[yaml[EXTEND].as_str().unwrap()].clone();
    } else {
        material = Material::default();
    }

    if yaml[COLOR] != BadValue {
        material.color = parse_color(&yaml[COLOR], definitions);
    }

    if yaml["pattern"] != BadValue {
        material.pattern = Some(parse_pattern(&yaml["pattern"], definitions));
    }

    if yaml["ambient"] != BadValue {
        material.ambient = parse_f64(&yaml["ambient"]);
    }

    if yaml["diffuse"] != BadValue {
        material.diffuse = parse_f64(&yaml["diffuse"]);
    }

    if yaml["specular"] != BadValue {
        material.specular = parse_f64(&yaml["specular"]);
    }

    if yaml["shininess"] != BadValue {
        material.shininess = parse_f64(&yaml["shininess"]);
    }

    if yaml["reflective"] != BadValue {
        material.reflectiveness = parse_f64(&yaml["reflective"]);
    }

    if yaml["transparency"] != BadValue {
        material.transparency = parse_f64(&yaml["transparency"]);
    }

    if yaml["refractive-index"] != BadValue {
        material.refractive_index = parse_f64(&yaml["refractive-index"]);
    }

    return material;
}

fn parse_pattern(yaml: &Yaml, definitions: &Definitions) -> Arc<dyn Pattern> {
    let colors = yaml["colors"].as_vec().unwrap();
    let color_a = parse_color(colors.get(0).unwrap(), definitions);
    let color_b = parse_color(colors.get(1).unwrap(), definitions);
    let maybe_transformation = match &yaml[TRANSFORMATION] {
        BadValue => None,
        _ => Some(parse_transformation(&yaml[TRANSFORMATION], definitions)),
    };
    return match &yaml[TYPE] {
        Yaml::String(value) => match value.as_str() {
            "stripes" => {
                let mut pattern = StripePattern::new(color_a, color_b);
                if let Some(transformation) = maybe_transformation {
                    pattern.set_transformation(transformation);
                }
                Arc::new(pattern)
            }
            "gradient" => {
                let mut pattern = GradientPattern::new(color_a, color_b);
                if let Some(transformation) = maybe_transformation {
                    pattern.set_transformation(transformation);
                }
                Arc::new(pattern)
            }
            "rings" => {
                let mut pattern = RingPattern::new(color_a, color_b);
                if let Some(transformation) = maybe_transformation {
                    pattern.set_transformation(transformation);
                }
                Arc::new(pattern)
            }
            "checkers" => {
                let mut pattern = CheckerPattern::new(color_a, color_b);
                if let Some(transformation) = maybe_transformation {
                    pattern.set_transformation(transformation);
                }
                Arc::new(pattern)
            }
            _ => panic!("Incorrect pattern type"),
        },
        _ => panic!("Incorrect pattern type"),
    };
}

fn parse_transformation_definition(yaml: &Yaml, definitions: &Definitions) -> Transformation {
    let mut transformation = transformations::IDENTITY;
    let yaml = &yaml[VALUE];

    for transform in yaml.clone().into_iter() {
        match transform {
            Yaml::String(ref value) => {
                transformation = transformation * definitions.transformations[&value.clone()];
            }
            Yaml::Array(ref values) => match values[0].as_str().unwrap() {
                "scale" => {
                    let vals = parse_array_of_3(&values[1..4]);
                    transformation =
                        transformations::scaling(vals[0], vals[1], vals[2]) * transformation;
                }
                "translate" => {
                    let vals = parse_array_of_3(&values[1..4]);
                    transformation =
                        transformations::translation(vals[0], vals[1], vals[2]) * transformation;
                }
                "rotate-x" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_x(vals) * transformation;
                }
                "rotate-y" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_y(vals) * transformation;
                }
                "rotate-z" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_z(vals) * transformation;
                }
                _ => {}
            },
            _ => {}
        }
    }
    return transformation;
}

fn parse_transformation(yaml: &Yaml, definitions: &Definitions) -> Transformation {
    let mut transformation = transformations::IDENTITY;

    for transform in yaml.clone().into_iter() {
        match transform {
            Yaml::String(ref value) => {
                transformation = transformation * definitions.transformations[&value.clone()];
            }
            Yaml::Array(ref values) => match values[0].as_str().unwrap() {
                "scale" => {
                    let vals = parse_array_of_3(&values[1..4]);
                    transformation =
                        transformations::scaling(vals[0], vals[1], vals[2]) * transformation;
                }
                "translate" => {
                    let vals = parse_array_of_3(&values[1..4]);
                    transformation =
                        transformations::translation(vals[0], vals[1], vals[2]) * transformation;
                }
                "rotate-x" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_x(vals) * transformation;
                }
                "rotate-y" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_y(vals) * transformation;
                }
                "rotate-z" => {
                    let vals = parse_f64(&values[1]);
                    transformation = transformations::rotation_z(vals) * transformation;
                }
                _ => {}
            },
            _ => {}
        }
    }
    return transformation;
}

fn parse_scene(yaml: &Yaml, definitions: &Definitions) -> (World, Camera) {
    let mut world = World::new(Vec::new(), Vec::new());
    let mut camera: Camera = Camera {
        horizontal_size: 0,
        vertical_size: 0,
        field_of_view: 0.0,
        half_width: 0.0,
        half_height: 0.0,
        pixel_size: 0.0,
        transformation: Default::default(),
    };
    for entry in yaml.clone().into_iter() {
        if let Yaml::String(name) = &entry[ADD] {
            match name.as_str() {
                "camera" => {
                    let horizontal_size = parse_f64(&entry["width"]) as u32;
                    let vertical_size = parse_f64(&entry["height"]) as u32;
                    let fov = parse_f64(&entry["field-of-view"]);
                    let from = parse_array_of_3(entry["from"].as_vec().unwrap());
                    let from = Point::new(from[0], from[1], from[2]);
                    let to = parse_array_of_3(entry["to"].as_vec().unwrap());
                    let to = Point::new(to[0], to[1], to[2]);
                    let up = parse_array_of_3(entry["up"].as_vec().unwrap());
                    let up = Vector::new(up[0], up[1], up[2]);
                    camera = Camera::new(horizontal_size, vertical_size, fov);
                    camera.transformation = transformations::view_transform(from, to, up);
                }
                "light" => {
                    let position = parse_array_of_3(entry["at"].as_vec().unwrap());
                    let position = Point::new(position[0], position[1], position[2]);
                    let intensity = parse_array_of_3(entry["intensity"].as_vec().unwrap());
                    let intensity = Color::new(intensity[0], intensity[1], intensity[2]);
                    world.lights.push(Light::new(position, intensity));
                }
                "plane" => {
                    let material = parse_material(&entry["material"], definitions);
                    let transformation = parse_transformation(&entry[TRANSFORMATION], definitions);
                    world
                        .shapes
                        .push(Box::new(Plane::new(material, transformation)));
                }
                "sphere" => {
                    let material = parse_material(&entry["material"], definitions);
                    let transformation = parse_transformation(&entry[TRANSFORMATION], definitions);
                    world
                        .shapes
                        .push(Box::new(Sphere::new(material, transformation)));
                }
                "cube" => {
                    let material = parse_material(&entry["material"], definitions);
                    let transformation = parse_transformation(&entry[TRANSFORMATION], definitions);
                    world
                        .shapes
                        .push(Box::new(Cube::new(material, transformation)));
                }
                "cone" => {
                    let material = parse_material(&entry["material"], definitions);
                    let transformation = parse_transformation(&entry[TRANSFORMATION], definitions);
                    world.shapes.push(Box::new(Cone {
                        material,
                        transformation,
                        ..Default::default()
                    }));
                }
                "cylinder" => {
                    let material = parse_material(&entry["material"], definitions);
                    let transformation = parse_transformation(&entry[TRANSFORMATION], definitions);
                    world.shapes.push(Box::new(Cylinder {
                        material,
                        transformation,
                        ..Default::default()
                    }));
                }
                _ => (),
            }
        }
    }
    return (world, camera);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use yaml_rust::{Yaml, YamlLoader};

    fn parse_yaml(value: &str) -> Yaml {
        return YamlLoader::load_from_str(value).unwrap().remove(0);
    }

    #[rstest]
    #[case("1", 1)]
    #[case("1.0", 1)]
    #[case(".0", 0)]
    #[case("0.", 0)]
    #[case("0", 0)]
    fn parse_f64_from_yaml(#[case] string: &str, #[case] expected: impl Into<f64>) {
        let yaml = parse_yaml(string);
        let value = parse_f64(&yaml);
        assert_eq!(value, expected.into());
    }

    #[rstest]
    #[case("[0.0, .0, 1.0]", [0.0, 0.0, 1.0])]
    #[case("[.0, 1.0, 1]", [0.0, 1.0, 1.0])]
    #[case("[10, 0, 1]", [10.0, 0.0, 1.0])]
    fn parse_array_of_3_from_yaml(#[case] string: &str, #[case] expected: [f64; 3]) {
        let yaml = parse_yaml(string);
        let value = parse_array_of_3(yaml.as_vec().unwrap());
        assert_eq!(value, expected);
    }
}
