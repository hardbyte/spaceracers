use anyhow::anyhow;
use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Map {
    pub name: String,
    pub size: Vec2,
    pub gravity: f32,
    pub obstacles: Vec<VectorObject>,
    pub start_regions: Vec<VectorObject>,
    pub finish_regions: Vec<VectorObject>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VectorObject {
    pub position: Vec2,
    pub polygon: Vec<Vec2>,
}

pub fn load_all_maps() -> HashMap<String, Map> {
    let mut maps = HashMap::new();
    maps.insert(
        "default_map".to_string(),
        Map {
            name: "default_map".to_string(),
            size: Vec2::new(900.0, 640.0),
            gravity: 0.0,
            obstacles: vec![VectorObject {
                position: Vec2 { x: 0.0, y: 0.0 },
                polygon: vec![
                    Vec2::new(-150.0, 100.0),
                    Vec2::new(-75.0, 100.0),
                    Vec2::new(-75.0, 175.0),
                    Vec2::new(-150.0, 175.0),
                    Vec2::new(-150.0, 100.0),
                ],
            }],
            start_regions: vec![VectorObject {
                position: Vec2 { x: 0.0, y: 0.0 },
                polygon: vec![],
            }],
            finish_regions: vec![VectorObject {
                position: Vec2 { x: 100.0, y: 100.0 },
                polygon: vec![],
            }],
        },
    );

    // TODO load all maps from tmx files in the maps directory
    // Use asset server
    let map = load_tiled_map("spacerace-server/assets/maps/test.tmx").expect("Failed to load map");
    maps.insert(map.name.clone(), map);

    maps
}

pub fn load_map(filename: &str) -> Option<Map> {
    // TODO only load all map data once, or make it lazy load
    let maps = load_all_maps();
    maps.get(filename).map(|m| m.clone())
}

fn load_tiled_map(filename: &str) -> anyhow::Result<Map> {
    let mut loader = tiled::Loader::new();
    let raw_map = loader.load_tmx_map(filename)?;

    let layer = raw_map
        .layers()
        .find_map(|l| l.as_object_layer())
        .ok_or(anyhow!("no object layer found in map"))?;

    // load map name from properties
    let map_name = raw_map
        .properties
        .get("name")
        .map(|prop| match prop {
            tiled::PropertyValue::StringValue(prop) => prop.clone(),
            _ => panic!("Unexpected map layer type"),
        })
        .unwrap_or_else(|| "tiled".to_string());

    let map_width = raw_map.width * raw_map.tile_width;
    let map_height = raw_map.height * raw_map.tile_height;

    // Load gravity from properties (default to 0.0 if not found)
    let gravity = raw_map
        .properties
        .get("gravity")
        .and_then(|prop| match prop {
            tiled::PropertyValue::FloatValue(f) => Some(f.clone()),
            _ => None,
        })
        .unwrap_or(0.0f32);

    let mut map = Map {
        name: map_name,
        size: Vec2::new(map_width as f32, map_height as f32),
        gravity,
        obstacles: vec![],
        finish_regions: vec![],
        start_regions: vec![],
    };

    for object in layer.object_data() {
        if let tiled::ObjectShape::Polygon { points } = &object.shape {
            let mut polygon: Vec<Vec2> = points
                .iter()
                // Invert Y-coordinates to fix from Tiled to game coordinates
                .map(|&(x, y)| Vec2::new(x, -y))
                //.map(Vec2::from)
                .collect();

            // Connect the last point back to the first to complete the shape
            let first = points.first().unwrap();
            polygon.push(Vec2 {
                x: first.0,
                y: -first.1,
            });

            let map_object = VectorObject {
                position: Vec2 {
                    x: object.x - (map_width as f32/ 2.0),
                    y: -(object.y) + (map_height as f32 / 2.0),
                },
                polygon,
            };

            match object.user_type.as_str() {
                "finish" => {
                    tracing::info!("Found finish region");
                    map.finish_regions.push(map_object);
                }
                "start" => {
                    map.start_regions.push(map_object);
                }
                // By default all other polygon objects are obstacles
                _ => {
                    map.obstacles.push(map_object);
                }
            }
        }
    }

    Ok(map)
}
