# sim-3d
A bevy app exploring 3d simulation using navmesh. The core of this demo uses `bevy 3d`, `bevy_egui`, and `vleue_navigator` for mesh pathing and navigation of the agent. The mesh and terrain are made in `Blender`.

<img width="1273" alt="Screenshot 2024-04-20 at 2 24 03 PM" src="https://github.com/RaminKav/sim-3d/assets/5355774/a78ccd9b-f0e2-4195-b40b-677a52135f55">

## Controls
- Move camera using `W A S D` , `Space`, and `Shift`
- Toggle navmesh view using `M`
- Toggle camera movement and cursor movement (for use in egui) using `ESC`
- `LEFT click` to spawn a target destination anywhere

## EGUI
The left EGUI panel allows for selecting which targets to queue for the agent. The `Simulate` button begins the simulation: the agent will move to each target one by one.


This is a very rough WIP demo, many features are missing, and edge-case bugs may or may not exist. 
