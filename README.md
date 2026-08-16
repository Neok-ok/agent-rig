# agent-rig (increment 1)

Agent-native scene file + physics inspect + headless PNG. One command writes a JSON scene an agent can author, steps a real physics world, dumps body state and contacts, and renders the post-step frame with a small CPU Cook-Torrance raytracer plus procedural IBL (spheres and boxes). No GPU. No Three.js in the engine.

## One command

```bash
cd /workspace/agent-rig && ./scripts/increment1.sh
```

Writes `artifacts/scene.json`, `artifacts/physics.json`, and `artifacts/frame.png` (our renderer, 800x450).

## Three.js baseline (comparison only)

```bash
cd /workspace/agent-rig && ./scripts/threejs-baseline.sh
```

Writes `artifacts/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + one directional, no environment map, no tonemap). Compare that file with `artifacts/frame.png`.

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

Covers: parse the demo scene; after stepping, the ball has dropped and contacts the ground (or sits at rest height); render is an 800x450 PNG larger than 1KB and not a solid color; the increment-1 path writes all three artifact files; the Three.js baseline PNG can be produced and is a real image.
