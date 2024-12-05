use std::collections::HashMap;
use bevy::prelude::Vec2;

pub struct Map {
    pub name: String,
    pub gravity: f32,
    pub obstacles: Vec<Obstacle>,
}

pub struct Obstacle {
    pub position: Vec2,
    pub size: Vec2,
}

pub fn load_maps() -> HashMap<String, Map> {

    let mut maps = HashMap::new();
    maps.insert("Gravity Map".to_string(), Map {
        name: "Gravity Map".to_string(),
        gravity: 9.8,
        obstacles: vec![
        // Ground
        Obstacle {
            position: Vec2::new(-180.0, -300.0),
            size: Vec2::new(1400.0, 10.0),
        },
        Obstacle {
            position: Vec2::new(-200.0, -200.0),
            size: Vec2::new(50.0, 50.0),
        },
        Obstacle {
            position: Vec2::new(0.0, 200.0),
            size: Vec2::new(50.0, 50.0),
        },
        ],
    });
    maps.insert("default_map".to_string(), Map {
        name: "default_map".to_string(),
        gravity: 0.0,
        obstacles: vec![Obstacle {
            position: Vec2::new(-150.0, 100.0),
            size: Vec2::new(75.0, 75.0),
        }],
    });

    maps
}
