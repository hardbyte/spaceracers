use anyhow::anyhow;
use bevy::prelude::Vec2;
use std::collections::HashMap;
use std::path::PathBuf;
use tiled_json_rs as tiled;

pub struct Map {
    pub name: String,
    pub gravity: f32,
    pub obstacles: Vec<Obstacle>,
}

pub struct Obstacle {
    pub polygon: Vec<Vec2>,
}

pub fn load_maps() -> HashMap<String, Map> {
    let mut maps = HashMap::new();
    maps.insert(
        "default_map".to_string(),
        Map {
            name: "default_map".to_string(),
            gravity: 0.0,
            obstacles: vec![Obstacle {
                polygon: vec![
                    Vec2::new(-150.0, 100.0),
                    Vec2::new(-75.0, 100.0),
                    Vec2::new(-75.0, 175.0),
                    Vec2::new(-150.0, 175.0),
                    Vec2::new(-150.0, 100.0),
                ],
            }],
        },
    );

    let map = load_map_from_tiled("../test.json").unwrap();
    maps.insert(map.name.clone(), map);

    maps
}

fn load_map_from_tiled(filename: &str) -> anyhow::Result<Map> {
    let raw_map = tiled::Map::load_from_file(&PathBuf::from(filename))?;

    // XXX load name from properties
    // XXX load gravity from properties

    let mut map = Map {
        name: "tiled".to_string(),
        gravity: 0.,
        obstacles: vec![],
    };

    let objects =
        find_first_object_layer(raw_map).ok_or(anyhow!("no object layer found in map"))?;
    for object in objects {
        tracing::warn!("found object {:?} {:?}", object.id, object.object_type);
        if let tiled::ObjectType::Polygon(points) = &object.object_type {
            tracing::warn!("object {:?} is polygon", object.id);
            let mut polygon = Vec::new();
            for point in points {
                polygon.push(Vec2 {
                    // XXX loading as i32 is a problem?
                    x: point.x as f32,
                    y: point.y as f32,
                })
            }
            map.obstacles.push(Obstacle { polygon });
        }
    }

    tracing::warn!("map has {} obstacles", map.obstacles.len());

    Ok(map)
}

fn find_first_object_layer(raw_map: tiled::Map) -> Option<Vec<tiled::Object>> {
    for layer in raw_map.layers {
        if let tiled::LayerType::ObjectGroup(objects) = &layer.layer_type {
            // TODO: avoid clone
            return Some(objects.objects.clone());
        }
    }
    None
}
