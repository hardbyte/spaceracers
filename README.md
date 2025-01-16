
# SpaceRaceRS



Run the game with:

```shell
cargo run
```

Enable logging by setting the `RUST_LOG` environment variable. For example:

```
RUST_LOG=warn,spaceracers=debug
```

By default, there is no UI, but you can enable it with the `ui` feature:

```shell
cargo run --bin spacerace-server --features ui
```

Or run with wayland support:

```shell
RUST_LOG=warn,spaceracer_server=debug cargo run --features ui,wayland
```

## HTTP Interface

Register a player in the lobby:

```http request
POST http://localhost:5000/lobby
Content-Type: application/json

{
  "name": "Player 1",
  "team": "The A Team",
  "password": "password"
}
```

Get the current state of the game:
```http request
GET http://localhost:5000/state
```

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
