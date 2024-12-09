
# SpaceRaceRS

Run the game with:

```shell
RUST_LOG=warn,spaceracers=debug cargo run --features ui
```

## Architecture

### Core Plugins:

**NetworkPlugin**: 
Handles HTTP endpoints and routes requests to/from the server.

**ControlPlugin**: 
Receives and stores control inputs (thrust/rotation) per player, to be applied by a physics update system.
Periodically updates a shared in-memory snapshot of the game state (positions, velocities) that the HTTP layer can serve.

**PhysicsPlugin**: 
Sets up Rapier and applies control forces each frame to update ship positions.

**GameLogicPlugin** (to extract from main): 
Manages game state transitions, including game start/end conditions and integrating with the lobby system.


