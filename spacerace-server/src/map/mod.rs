use anyhow::anyhow;
use bevy::prelude::Vec2;
use std::collections::HashMap;

pub struct Map {
    pub name: String,
    pub gravity: f32,
    pub obstacles: Vec<Obstacle>,
}

pub struct Obstacle {
    pub position: Vec2,
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
                position: Vec2 { x: 0.0, y: 0.0 },
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

    // TODO load all maps from tmx files in a directory
    let map = load_tiled_map("maps/test.tmx").unwrap();
    maps.insert(map.name.clone(), map);

    maps
}

fn load_tiled_map(filename: &str) -> anyhow::Result<Map> {
    let mut loader = tiled::Loader::new();
    let raw_map = loader.load_tmx_map(filename)?;

    let layer = raw_map
        .layers()
        .find_map(|l| l.as_object_layer())
        .ok_or(anyhow!("no object layer found in map"))?;

    // TODO loaded objects are upside down in the game!
    // TODO load map name from properties
    // TODO load gravity from properties
    // TODO use object properties for start and end regions

    let mut map = Map {
        name: "tiled".to_string(),
        gravity: 0.,
        obstacles: vec![],
    };

    for object in layer.object_data() {
        if let tiled::ObjectShape::Polygon { points } = &object.shape {
            let mut polygon: Vec<Vec2> = points.iter().cloned().map(Vec2::from).collect();

            // Connect the last point back to the first to complete the shape
            let first = points.first().unwrap();
            polygon.push(Vec2 {
                x: first.0,
                y: first.1,
            });

            map.obstacles.push(Obstacle {
                position: Vec2 {
                    x: object.x,
                    y: object.y,
                },
                polygon,
            });
        }
    }

    Ok(map)
}
