
# SpaceRaceRS

SpaceRaceRS is a multiplayer game where players pilot spaceships in real-time across varied 2D maps. Movement and collisions
are governed by a physics engine, and the game’s server handles all core logic—position updates, collision detection, 
race progression, and more. Client(s) communicate with the server via simple HTTP endpoints to register for a match,
send control inputs (thrust, rotation), and receive the latest world state.

Features

- Physics-based movement: Ships drift and collide with obstacles using the Rapier 2D physics engine.
- Lobby system: Players join a lobby, and the server automatically transitions to Active state when a game is ready.
- Multiple maps: Tiled-based map loading with support for obstacles, start zones, finish zones, etc.
- Extensible: The server is modular, allowing a variety of front-ends (e.g., web clients, custom desktop GUIs).



## Usage

Run the HTTP server with a simple UI:

```shell
spaceracer_server=debug cargo run --bin spacerace-server --features ui
```

Or run with wayland support:

```shell
RUST_LOG=warn,spaceracer_server=debug cargo run --features ui,wayland
```


# HTTP Interface

### Lobby Endpoint 

Register a player in the next game by POSTing to the `/lobby` endpoint.

```http request
POST http://localhost:5000/lobby
Content-Type: application/json

{
  "name": "Player Name",
  "team": "The A Team",
  "password": "password"
}
```

Response will be something like:

```json
{
  "player_id": "Player 1",
  "game_id": "c5d43c81-bca2-4c2f-aa8b-35d8e5a9ff72",
  "map": "Aga"
}
```

### State Endpoint

Retrieve the current state of the game (positions, velocities, etc.):

```http request
GET http://localhost:5000/state
```

### Control Endpoint

Control your ship:
```http request
POST http://localhost:5000/control
Content-Type: application/json

{
  "password": "password",
  "thrust": 1,
  "rotation": 0
}
###
```



## Example Client

A simple client is provided in the `spacerace-client` directory. To run it, use the following command:

```shell
PLAYER_TEAM=Humans PLAYER_NAME=Brian SPACERACERS_SERVER=http://10.1.0.179:5000 cargo run --package spacerace-client
```

# Creating or Editing a Map

SpaceRaceRS uses Tiled `.tmx maps`. To create or modify a map:

### Install Tiled
Download Tiled and create or open an existing .tmx file.

### Add Object Layer
Create an “Object Layer” for obstacles, start regions, or finish regions.

### Mark Objects
    
user_type = "start" for polygons that represent start zones.
user_type = "finish" for polygons that represent finish zones.
Any other polygons become obstacles by default.

### Properties

skin (optional): A background image path, e.g., "assets/images/background.png".
ship (optional): The sprite path for ships, e.g., "my_ship.png".
gravity (optional): A float specifying downward force.

Positioning
The map is centered on (0,0). Tiled’s default origin is top-left, so the loader automatically re-centers objects.

The final bounding is map.size.x by map.size.y.

### Integrate

Add your .tmx map path to load_all_maps() in map.rs.
Reference it in your Lobby or game creation logic.