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

pub fn load_maps() -> Vec<Map> {
    vec![
        Map {
            name: "Gravity Map".to_string(),
            gravity: 9.8,
            obstacles: vec![Obstacle {
                position: Vec2::new(100.0, 200.0),
                size: Vec2::new(50.0, 50.0),
            }],
        },
        Map {
            name: "No Gravity Map".to_string(),
            gravity: 0.0,
            obstacles: vec![Obstacle {
                position: Vec2::new(-150.0, 100.0),
                size: Vec2::new(75.0, 75.0),
            }],
        },
    ]
}
