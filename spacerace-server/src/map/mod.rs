use anyhow::anyhow;
use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadedFolder},
    prelude::*,
    reflect::TypePath,
};

use crate::{app_state::AppState, game_logic::ServerState};

#[derive(Default)]
pub struct MapAssetLoader;

impl AssetLoader for MapAssetLoader {
    type Asset = Map;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let res_reader = MemoryReader { bytes };

        // TODO: eliminate Loader's cache as bevy already as this covered
        let mut loader = tiled::Loader::with_reader(res_reader);

        // Note: the path here is only used for error reporting during parsing
        let raw_map = loader.load_tmx_map(load_context.path())?;

        let map = Map::new(raw_map)?;
        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

struct MemoryReader {
    bytes: Vec<u8>,
}

impl tiled::ResourceReader for MemoryReader {
    type Resource = Cursor<Vec<u8>>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        // TODO: it would be nice to avoid this clone
        Ok(Cursor::new(self.bytes.clone()))
    }
}

#[derive(Resource, Default)]
pub struct MapsFolder(Handle<LoadedFolder>);

pub fn load_maps(asset_server: ResMut<AssetServer>, mut commands: Commands) {
    let folder_handle: Handle<LoadedFolder> = asset_server.load_folder("maps");
    commands.insert_resource(MapsFolder(folder_handle));
}

#[derive(Debug, Clone)]
pub struct NamedMapId(pub String, pub AssetId<Map>);

pub fn check_maps_loaded(
    app_state: Res<AppState>,
    mut next_state: ResMut<NextState<ServerState>>,
    mut events: EventReader<AssetEvent<Map>>,
    maps: Res<Assets<Map>>,
    asset_server: Res<AssetServer>,
    maps_folder: Res<MapsFolder>,
) {
    // Advance the `AppState` once the maps are loaded
    let mut loaded = false;
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let map = maps.get(*id).unwrap();
            tracing::info!("map loaded: {:?}", map.name);
            app_state.add_map(NamedMapId(map.name.clone(), *id));
            loaded = true;
        }
    }

    if loaded
        && asset_server
            .recursive_dependency_load_state(maps_folder.0.id())
            .is_loaded()
    {
        tracing::info!("All maps loaded!");
        next_state.set(ServerState::Inactive); // Assets are loaded, let the game proceeed
    }
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub skin_path: Option<String>,
    pub ship_path: Option<String>,
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

impl Map {
    fn new(raw_map: tiled::Map) -> anyhow::Result<Map> {
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

        let skin_path: Option<String> =
            raw_map.properties.get("skin").and_then(|prop| match prop {
                tiled::PropertyValue::StringValue(prop) => Some(prop.clone()),
                _ => None,
            });
        tracing::debug!("Map skin path: {:?}", skin_path);

        let ship_path: Option<String> =
            raw_map.properties.get("ship").and_then(|prop| match prop {
                tiled::PropertyValue::StringValue(prop) => Some(prop.clone()),
                _ => None,
            });
        tracing::debug!("Ship path: {:?}", ship_path);

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
            skin_path,
            ship_path,
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
                        x: object.x - (map_width as f32 / 2.0),
                        y: -(object.y) + (map_height as f32 / 2.0),
                    },
                    polygon,
                };

                match object.user_type.as_str() {
                    "finish" => {
                        tracing::debug!("Found finish region");
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
}
