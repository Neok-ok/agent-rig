# agent-rig (increments 1–5)

Agent-native scene file + physics inspect + headless PNG. One command writes a JSON scene an agent can author, steps a real physics world, dumps body state and contacts, and renders the post-step frame with a small CPU Cook-Torrance raytracer plus procedural IBL (spheres, boxes, and triangle meshes, with optional albedo textures). No GPU. No Three.js in the engine.

## Increment 1

Ball falling onto a ground box.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment1.sh
```

Writes `artifacts/scene.json`, `artifacts/physics.json`, and `artifacts/frame.png` (our renderer, 800x450).

Equivalent CLI:

```bash
agent-rig demo --out artifacts
# `--demo` still works for increment 1:
agent-rig --demo --out artifacts
```

### Three.js baseline (comparison only)

```bash
cd /workspace/agent-rig && ./scripts/threejs-baseline.sh
```

Writes `artifacts/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + one directional, no environment map, no tonemap). Compare that file with `artifacts/frame.png`.

## Increment 2

Multi-body scene: metal ball, rough dielectric crate, metal stopper, ground. The ball is given an initial velocity so it strikes the crate (body-body contact). Same JSON schema as increment 1.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment2.sh
```

Writes:

- `artifacts/increment2/scene.json` — authored scene
- `artifacts/increment2/physics.json` — post-step dump (poses, velocities, contacts)
- `artifacts/increment2/frame.png` — our renderer, 800x450, post-step poses

### CLI step / render

```bash
agent-rig step artifacts/increment2/scene.json --out artifacts/increment2/physics.json --steps 120
agent-rig render artifacts/increment2/scene.json --out artifacts/increment2/frame.png
# apply dump poses when rendering:
agent-rig render artifacts/increment2/scene.json --physics artifacts/increment2/physics.json --out artifacts/increment2/frame.png
```

`step` defaults to 90 steps. The `increment2` subcommand uses 120 so the ball has time to hit the crate.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment2-threejs.sh
```

Writes `artifacts/increment2/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads the increment-2 scene JSON and applies `physics.json` poses.


## Increment 3

Ramp scene: static ground, a static box rotated ~30° about Z as a ramp, a metal ball at the top, and a rough dielectric crate at the bottom. Physics is stepped over time; a PNG is emitted every stride (default 10 frames, stride 20 → 180 steps). Same JSON schema and light as increment 2.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment3.sh
```

Writes:

- `artifacts/increment3/scene.json` — authored ramp scene
- `artifacts/increment3/trajectory.json` — snapshots (positions, velocities, contacts) at each emitted frame
- `artifacts/increment3/frame.png` — our renderer, 800x450, last frame
- `artifacts/increment3/frames/frame_00.png` … — one PNG per snapshot

### CLI

```bash
agent-rig sim artifacts/increment3/scene.json --frames 10 --out artifacts/increment3
```

`--frames` is the number of PNGs. Internally the world is stepped `stride` times between frames (default 20). `demo`, `increment2`, `step`, and `render` are unchanged.

### Three.js baseline (same last-frame pose)

```bash
cd /workspace/agent-rig && ./scripts/increment3-threejs.sh
```

Writes `artifacts/increment3/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads the increment-3 scene JSON and applies the last trajectory pose.


## Increment 4

Triangle-mesh body: a faceted crystal/rock (OBJ) sits on the ground with a metal sphere rolling into it, plus a rough crate. Same light as before. Physics uses a convex-hull collider on the mesh; the dump records `collider` per body.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment4.sh
```

Writes:

- `artifacts/increment4/scene.json` — authored scene (mesh + primitives)
- `artifacts/increment4/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment4/frame.png` — our renderer, 800x450, post-step poses
- `artifacts/increment4/threejs-frame.png` — stock Three.js, same poses

### CLI

```bash
agent-rig increment4 --out artifacts/increment4
agent-rig sim artifacts/increment4/scene.json --out artifacts/increment4-sim
agent-rig render artifacts/increment4/scene.json --physics artifacts/increment4/physics.json --out artifacts/increment4/frame.png
```

`sim` / `render` load the OBJ from `shape.path`.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment4-threejs.sh
```

Writes `artifacts/increment4/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads the increment-4 scene JSON (including the OBJ mesh) and applies `physics.json` poses.


## Increment 5

Image albedo on the mesh: the faceted rock uses `textures/rock.png` (orange/cyan checker) via `material.albedo_map`. Same primitives and light as increment 4. The CPU renderer samples the map with triangle UVs; primitives stay untextured.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment5.sh
```

Writes:

- `artifacts/increment5/scene.json` — authored scene (textured mesh + primitives)
- `artifacts/increment5/physics.json` — post-step dump
- `artifacts/increment5/frame.png` — our renderer, 800x450, post-step poses
- `artifacts/increment5/threejs-frame.png` — stock Three.js, same poses

### CLI

```bash
agent-rig increment5 --out artifacts/increment5
agent-rig sim artifacts/increment5/scene.json --out artifacts/increment5-sim
agent-rig render artifacts/increment5/scene.json --physics artifacts/increment5/physics.json --out artifacts/increment5/frame.png
```

`sim` / `render` load `albedo_map` when present (path relative to the repo root or the scene file).

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment5-threejs.sh
```

Writes `artifacts/increment5/threejs-frame.png` (stock MeshStandardMaterial + `map` from `albedo_map`, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450).

## Scene format

JSON object with `camera`, `lights`, and `bodies`.

- `camera`: `position`, `look_at`, `fov_y_deg`
- `lights`: `type: "directional"` with `direction`, `color`, `intensity`
- `bodies`: `id`, `shape` (`box` + full `size` xyz, `sphere` + `radius`, or `mesh` + `path` + `collider`), `position`, optional `rotation_wxyz` (default `[1,0,0,0]`), `mass` (0 = static), optional `linear_velocity`, `material` (`albedo`, `roughness`, `metallic`, optional `albedo_map` PNG path)
- mesh `path` and `albedo_map` are relative to the repo root (or the scene file). `collider` is `convex_hull` or `trimesh`. If `albedo_map` is set, the sampled texel replaces albedo on that body.

Gravity is `[0, -9.81, 0]`.

## Tests

```bash
cd /workspace/agent-rig && cargo test
```

Increment 1: parse the demo scene; after stepping, the ball has dropped and contacts the ground (or sits at rest height); render is an 800x450 PNG larger than 1KB and not a solid color; the increment-1 path writes all three artifact files; the Three.js baseline PNG can be produced and is a real image.

Increment 2: scene parses with ≥3 non-ground bodies, a sphere and a box, metal and rough dielectric; after stepping, at least two dynamic bodies have moved and the dump has contacts (including a non-ground pair); `step` / `render` write the named files; `run_increment2` writes the three artifacts.

Increment 3: scene has a ramp (rotated static box); `sim` writes ≥8 frame PNGs and a trajectory dump with one snapshot per frame; a dynamic body moves across the trajectory; `increment3.sh` writes scene + trajectory + last-frame + frames/.

Increment 4: an OBJ triangle mesh loads (vertex/triangle counts > 0, not a box/sphere); after stepping there is a contact involving the mesh body; `increment4` writes scene + physics dump + our PNG.

Increment 5: a non-solid albedo PNG loads; the increment-5 mesh material points `albedo_map` at it; `increment5` writes scene + physics dump + our PNG and the rendered mesh is not a single albedo.
