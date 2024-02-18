use ray_tracer::composites::{Camera, Material, World};
use ray_tracer::patterns::{CheckerPattern, GradientPattern, Pattern, RingPattern, StripePattern};
use ray_tracer::primitives::{transformations, Transformation};
use ray_tracer::primitives::{Color, Light, Point, Vector};
use ray_tracer::shapes::{Cone, Cube, Plane, Shape, Sphere};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::num::ParseFloatError;
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
const MATERIAL: &str = "material";
const TRANSFORMATION: &str = "transform";

#[derive(Clone, Debug)]
struct SceneParser {
    colors: HashMap<String, Color>,
    materials: HashMap<String, Material>,
    patterns: HashMap<String, Arc<dyn Pattern>>,
    transformations: HashMap<String, Transformation>,
}

impl SceneParser {
    pub fn new() -> Self {
        return Self {
            colors: HashMap::<String, Color>::new(),
            materials: HashMap::<String, Material>::new(),
            patterns: HashMap::<String, Arc<dyn Pattern>>::new(),
            transformations: HashMap::<String, Transformation>::new(),
        };
    }

    fn process_definitions(&mut self, yaml: &Yaml) -> Result<(), Box<dyn Error>> {
        for entry in yaml.clone() {
            if let Yaml::String(name) = &entry[DEFINE] {
                if name.ends_with("-color") {
                    let color = self.parse_color(&entry)?;
                    self.colors.insert(name.clone(), color);
                } else if name.ends_with("-material") {
                    let material = self.parse_material(&entry)?;
                    self.materials.insert(name.clone(), material);
                } else if name.ends_with("-transform") || name.ends_with("-object") {
                    let transformation = self.parse_transformation(&entry)?;
                    self.transformations.insert(name.clone(), transformation);
                }
            }
        }
        return Ok(());
    }

    fn parse_color(&self, yaml: &Yaml) -> Result<Color, ParseFloatError> {
        match yaml {
            Yaml::Hash(_) => {
                return self.parse_color(
                    &yaml[if yaml[COLOR] == BadValue {
                        VALUE
                    } else {
                        COLOR
                    }],
                );
            }
            Yaml::Array(color_values) => {
                let color_channels = parse_array_of_3(color_values)?;
                return Ok(Color::new(
                    color_channels[0],
                    color_channels[1],
                    color_channels[2],
                ));
            }
            Yaml::String(key) => {
                return Ok(self.colors[key]);
            }
            _ => {
                panic!("Incorrect color value");
            }
        }
    }

    fn parse_material(&self, yaml: &Yaml) -> Result<Material, Box<dyn Error>> {
        if let Yaml::String(name) = yaml {
            return Ok(self.materials[name].clone());
        }

        let mut material = if yaml[EXTEND] == BadValue {
            Material::default()
        } else {
            self.materials[yaml[EXTEND].as_str().unwrap()].clone()
        };

        let yaml = if yaml[VALUE] != BadValue {
            &yaml[VALUE]
        } else {
            yaml
        };
        if yaml[COLOR] != BadValue {
            material.color = self.parse_color(&yaml[COLOR])?;
        }

        if yaml["pattern"] != BadValue {
            material.pattern = Some(self.parse_pattern(&yaml["pattern"])?);
        }

        if yaml["ambient"] != BadValue {
            material.ambient = parse_f64(&yaml["ambient"])?;
        }

        if yaml["diffuse"] != BadValue {
            material.diffuse = parse_f64(&yaml["diffuse"])?;
        }

        if yaml["specular"] != BadValue {
            material.specular = parse_f64(&yaml["specular"])?;
        }

        if yaml["shininess"] != BadValue {
            material.shininess = parse_f64(&yaml["shininess"])?;
        }

        if yaml["reflective"] != BadValue {
            material.reflectiveness = parse_f64(&yaml["reflective"])?;
        }

        if yaml["transparency"] != BadValue {
            material.transparency = parse_f64(&yaml["transparency"])?;
        }

        if yaml["refractive-index"] != BadValue {
            material.refractive_index = parse_f64(&yaml["refractive-index"])?;
        }

        if let Yaml::Boolean(value) = yaml["casts-shadow"] {
            material.casts_shadow = value;
        }

        return Ok(material);
    }

    fn parse_pattern(&self, yaml: &Yaml) -> Result<Arc<dyn Pattern>, Box<dyn Error>> {
        let colors = yaml["colors"].as_vec().unwrap();
        let color_a = self.parse_color(&colors[0])?;
        let color_b = self.parse_color(&colors[1])?;
        let maybe_transformation = match yaml[TRANSFORMATION] {
            BadValue => None,
            _ => Some(self.parse_transformation(&yaml[TRANSFORMATION])?),
        };
        return match &yaml[TYPE] {
            Yaml::String(value) => match value.as_str() {
                "stripes" => {
                    let mut pattern = StripePattern::new(color_a, color_b);
                    if let Some(transformation) = maybe_transformation {
                        pattern.set_transformation(transformation);
                    }
                    Ok(Arc::new(pattern))
                }
                "gradient" => {
                    let mut pattern = GradientPattern::new(color_a, color_b);
                    if let Some(transformation) = maybe_transformation {
                        pattern.set_transformation(transformation);
                    }
                    Ok(Arc::new(pattern))
                }
                "rings" => {
                    let mut pattern = RingPattern::new(color_a, color_b);
                    if let Some(transformation) = maybe_transformation {
                        pattern.set_transformation(transformation);
                    }
                    Ok(Arc::new(pattern))
                }
                "checkers" => {
                    let mut pattern = CheckerPattern::new(color_a, color_b);
                    if let Some(transformation) = maybe_transformation {
                        pattern.set_transformation(transformation);
                    }
                    Ok(Arc::new(pattern))
                }
                _ => panic!("Incorrect pattern type"),
            },
            _ => panic!("Incorrect pattern type"),
        };
    }

    fn parse_transformation(&self, yaml: &Yaml) -> Result<Transformation, Box<dyn Error>> {
        let mut transformation = Transformation::IDENTITY;

        let yaml = if yaml[VALUE] != BadValue {
            &yaml[VALUE]
        } else {
            yaml
        };

        for transform in yaml.clone().into_iter() {
            match transform {
                Yaml::String(value) => {
                    transformation = transformation * self.transformations[&value.clone()];
                }
                Yaml::Array(values) => match values[0].as_str().unwrap() {
                    "scale" => {
                        let vals = parse_array_of_3(&values[1..])?;
                        transformation =
                            transformations::scaling(vals[0], vals[1], vals[2]) * transformation;
                    }
                    "translate" => {
                        let vals = parse_array_of_3(&values[1..])?;
                        transformation = transformations::translation(vals[0], vals[1], vals[2])
                            * transformation;
                    }
                    "rotate-x" => {
                        let value = parse_f64(&values[1])?;
                        transformation = transformations::rotation_x(value) * transformation;
                    }
                    "rotate-y" => {
                        let value = parse_f64(&values[1])?;
                        transformation = transformations::rotation_y(value) * transformation;
                    }
                    "rotate-z" => {
                        let value = parse_f64(&values[1])?;
                        transformation = transformations::rotation_z(value) * transformation;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        return Ok(transformation);
    }

    fn parse_material_and_transformation(
        &self,
        yaml: &Yaml,
    ) -> Result<(Material, Transformation), Box<dyn Error>> {
        let material = self.parse_material(&yaml[MATERIAL])?;
        let transformation = self.parse_transformation(&yaml[TRANSFORMATION])?;
        return Ok((material, transformation));
    }

    fn parse_scene(&self, yaml: &Yaml) -> Result<(World, Camera), Box<dyn Error>> {
        let mut world = World::new(Vec::new(), Vec::new());
        let mut camera = Camera::new(0, 0, 0);
        for entry in yaml.clone().into_iter() {
            if let Yaml::String(name) = &entry[ADD] {
                match name.as_str() {
                    "camera" => {
                        let horizontal_size = parse_f64(&entry["width"])? as u32;
                        let vertical_size = parse_f64(&entry["height"])? as u32;
                        let fov = parse_f64(&entry["field-of-view"])?;
                        let from = parse_array_of_3(entry["from"].as_vec().unwrap())?;
                        let from = Point::from(from);
                        let to = parse_array_of_3(entry["to"].as_vec().unwrap())?;
                        let to = Point::from(to);
                        let up = parse_array_of_3(entry["up"].as_vec().unwrap())?;
                        let up = Vector::from(up);
                        camera = Camera::new(horizontal_size, vertical_size, fov);
                        camera.set_transformation(transformations::view_transform(from, to, up));
                    }
                    "light" => {
                        let position = parse_array_of_3(entry["at"].as_vec().unwrap())?;
                        let position = Point::new(position[0], position[1], position[2]);
                        let intensity = parse_array_of_3(entry["intensity"].as_vec().unwrap())?;
                        let intensity = Color::new(intensity[0], intensity[1], intensity[2]);
                        world.lights.push(Light::new(position, intensity));
                    }
                    "plane" => {
                        let (material, transformation) =
                            self.parse_material_and_transformation(&entry)?;
                        world
                            .shapes
                            .push(Box::new(Plane::new(material, transformation)));
                    }
                    "sphere" => {
                        let (material, transformation) =
                            self.parse_material_and_transformation(&entry)?;
                        world
                            .shapes
                            .push(Box::new(Sphere::new(material, transformation)));
                    }
                    "cube" => {
                        let (material, transformation) =
                            self.parse_material_and_transformation(&entry)?;
                        world
                            .shapes
                            .push(Box::new(Cube::new(material, transformation)));
                    }
                    "cone" => {
                        let (material, transformation) =
                            self.parse_material_and_transformation(&entry)?;
                        let mut cone = Cone::default();
                        cone.material = material;
                        cone.set_transformation(transformation);
                        if let Yaml::Boolean(value) = &entry["closed"] {
                            cone.closed = *value;
                        }
                        if entry["max"] != BadValue {
                            cone.max = parse_f64(&entry["max"])?;
                        }
                        if entry["min"] != BadValue {
                            cone.min = parse_f64(&entry["min"])?;
                        }
                        world.shapes.push(Box::new(cone));
                    }
                    "cylinder" => {
                        let (material, transformation) =
                            self.parse_material_and_transformation(&entry)?;
                        let mut cylinder = Cone::default();
                        cylinder.material = material;
                        cylinder.set_transformation(transformation);
                        if let Yaml::Boolean(value) = entry["closed"] {
                            cylinder.closed = value;
                        }
                        if entry["max"] != BadValue {
                            cylinder.max = parse_f64(&entry["max"])?;
                        }
                        if entry["min"] != BadValue {
                            cylinder.min = parse_f64(&entry["min"])?;
                        }
                        world.shapes.push(Box::new(cylinder));
                    }
                    _ => (),
                }
            }
        }
        return Ok((world, camera));
    }
}

fn parse_f64(yaml: &Yaml) -> Result<f64, ParseFloatError> {
    match yaml {
        Yaml::Integer(value) => Ok(*value as f64),
        Yaml::Real(value) => value.parse(),
        _ => "".parse(),
    }
}

fn parse_array_of_3(slice: &[Yaml]) -> Result<[f64; 3], ParseFloatError> {
    let values = {
        let values: Result<Vec<_>, _> = slice.iter().take(3).map(parse_f64).collect();
        values?
    };
    return Ok([values[0], values[1], values[2]]);
}

fn load_file_to_yaml<P: AsRef<Path>>(path: P) -> Result<Yaml, Box<dyn Error>> {
    let file = fs::read_to_string(path)?;
    let mut docs = YamlLoader::load_from_str(&file)?;
    assert_eq!(docs.len(), 1, "Incorrect yaml format");
    return Ok(docs.remove(0));
}

pub fn load_scene_description<P: AsRef<Path>>(path: P) -> Result<(World, Camera), Box<dyn Error>> {
    let yaml = load_file_to_yaml(path)?;
    let mut scene_parser = SceneParser::new();

    scene_parser.process_definitions(&yaml)?;
    return scene_parser.parse_scene(&yaml);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
        let value = parse_f64(&yaml).unwrap();
        assert_eq!(value, expected.into());
    }

    #[rstest]
    #[case("[0.0, .0, 1.0]", [0.0, 0.0, 1.0])]
    #[case("[.0, 1.0, 1]", [0.0, 1.0, 1.0])]
    #[case("[10, 0, 1]", [10.0, 0.0, 1.0])]
    fn parse_array_of_3_from_yaml(#[case] string: &str, #[case] expected: [f64; 3]) {
        let yaml = parse_yaml(string);
        let value = parse_array_of_3(yaml.as_vec().unwrap()).unwrap();
        assert_eq!(value, expected);
    }
}
