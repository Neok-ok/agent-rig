# agent-rig (increments 1–2)

Agent-native scene file + physics inspect + headless PNG. One command writes a JSON scene an agent can author, steps a real physics world, dumps body state and contacts, and renders the post-step frame with a small CPU Cook-Torrance raytracer plus procedural IBL (spheres and boxes). No GPU. No Three.js in the engine.

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

## Scene format

JSON object with `camera`, `lights`, and `bodies`.

- `camera`: `position`, `look_at`, `fov_y_deg`
- `lights`: `type: "directional"` with `direction`, `color`, `intensity`
- `bodies`: `id`, `shape` (`box` + full `size` xyz, or `sphere` + `radius`), `position`, optional `rotation_wxyz` (default `[1,0,0,0]`), `mass` (0 = static), optional `linear_velocity`, `material` (`albedo`, `roughness`, `metallic`)

Gravity is `[0, -9.81, 0]`.

## Tests

```bash
cd /workspace/agent-rig && cargo test
```

Increment 1: parse the demo scene; after stepping, the ball has dropped and contacts the ground (or sits at rest height); render is an 800x450 PNG larger than 1KB and not a solid color; the increment-1 path writes all three artifact files; the Three.js baseline PNG can be produced and is a real image.

Increment 2: scene parses with ≥3 non-ground bodies, a sphere and a box, metal and rough dielectric; after stepping, at least two dynamic bodies have moved and the dump has contacts (including a non-ground pair); `step` / `render` write the named files; `run_increment2` writes the three artifacts.
