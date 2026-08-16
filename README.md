# agent-rig (increments 1–10)

Agent-native scene file + physics inspect + headless PNG. One command writes a JSON scene an agent can author, steps a real physics world, dumps body state and contacts, and renders the post-step frame with a small CPU Cook-Torrance raytracer plus procedural IBL (spheres, boxes, and triangle meshes from OBJ or a constrained glTF/GLB, with optional albedo textures and glTF pbrMetallicRoughness). No GPU. No Three.js in the engine.

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


## Increment 6

Two distinct triangle meshes on a plane: the existing textured rock (`meshes/rock.obj`) plus a wooden wedge / doorstop (`meshes/wedge.obj`). A metal ball rolls into the rock; a rough crate sits nearby. Same light as increment 5. Each mesh has a queryable collider (`convex_hull` on the rock, `trimesh` on the wedge); the physics dump records `collider` for both.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment6.sh
```

Writes:

- `artifacts/increment6/scene.json` — authored scene (two meshes + primitives)
- `artifacts/increment6/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment6/frame.png` — our renderer, 800x450, post-step poses
- `artifacts/increment6/threejs-frame.png` — stock Three.js, same poses

### CLI

```bash
agent-rig increment6 --out artifacts/increment6
agent-rig sim artifacts/increment6/scene.json --out artifacts/increment6-sim
agent-rig render artifacts/increment6/scene.json --physics artifacts/increment6/physics.json --out artifacts/increment6/frame.png
```

`sim` / `render` load both OBJ paths from the scene.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment6-threejs.sh
```

Writes `artifacts/increment6/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads both OBJ files from the increment-6 scene JSON and applies `physics.json` poses.


## Increment 7

Environment mesh as the ground: a shallow courtyard dish (`meshes/bowl.obj`) with a raised rim replaces the box plane. The textured rock and a metal ball sit in the dish. Physics uses a `trimesh` collider on the environment mesh so props rest on the floor; the dump records `collider` for that body.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment7.sh
```

Writes:

- `artifacts/increment7/scene.json` — authored scene (environment mesh + rock + ball)
- `artifacts/increment7/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment7/frame.png` — our renderer, 800x450, post-step poses
- `artifacts/increment7/threejs-frame.png` — stock Three.js, same poses

### CLI

```bash
agent-rig increment7 --out artifacts/increment7
agent-rig sim artifacts/increment7/scene.json --out artifacts/increment7-sim
agent-rig render artifacts/increment7/scene.json --physics artifacts/increment7/physics.json --out artifacts/increment7/frame.png
```

`sim` / `render` load the environment mesh from `shape.path`.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment7-threejs.sh
```

Writes `artifacts/increment7/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads the increment-7 scene JSON (bowl + rock OBJs) and applies `physics.json` poses.


## Increment 8

glTF mesh body in the increment-7 courtyard: a low-poly hexagonal pillar (`meshes/pillar.gltf`) sits in the dish with the textured rock and metal ball. `load_mesh` dispatches on extension (`.obj` / `.gltf` / `.glb`). Physics uses a `convex_hull` collider on the pillar; the dump records `collider` for that body and a contact involving it.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment8.sh
```

Writes:

- `artifacts/increment8/scene.json` — authored scene (bowl + rock + ball + glTF pillar)
- `artifacts/increment8/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment8/frame.png` — our renderer, 800x450, post-step poses
- `artifacts/increment8/threejs-frame.png` — stock Three.js, same poses

### CLI

```bash
agent-rig increment8 --out artifacts/increment8
agent-rig sim artifacts/increment8/scene.json --out artifacts/increment8-sim
agent-rig render artifacts/increment8/scene.json --physics artifacts/increment8/physics.json --out artifacts/increment8/frame.png
```

`sim` / `render` load the glTF path from `shape.path`.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment8-threejs.sh
```

Writes `artifacts/increment8/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450). Loads the increment-8 scene JSON (bowl + rock OBJs and the glTF pillar) and applies `physics.json` poses.


## Increment 9

Same courtyard as increment 8. The hexagonal pillar's look comes from `pbrMetallicRoughness` in `meshes/pillar.gltf` (`baseColorFactor` copper `[0.85, 0.45, 0.18, 1]`, `metallicFactor` 0.85, `roughnessFactor` 0.25). Scene JSON on that body is a dull-gray fallback used only when the glTF primitive has no material.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment9.sh
```

Writes:

- `artifacts/increment9/scene.json` — authored scene (bowl + rock + ball + glTF pillar, gray JSON fallback)
- `artifacts/increment9/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment9/frame.png` — our renderer, 800x450, post-step poses (glTF copper + IBL)
- `artifacts/increment9/threejs-frame.png` — stock Three.js, same poses (glTF factor on MeshStandardMaterial, no IBL/tonemap)

### CLI

```bash
agent-rig increment9 --out artifacts/increment9
agent-rig sim artifacts/increment9/scene.json --out artifacts/increment9-sim
agent-rig render artifacts/increment9/scene.json --physics artifacts/increment9/physics.json --out artifacts/increment9/frame.png
```

`sim` / `render` load glTF materials via the shared loader.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment9-threejs.sh
```

Writes `artifacts/increment9/threejs-frame.png` (stock MeshStandardMaterial using the glTF `baseColorFactor` / map, ambient 0.25 + directional, shadows on, no env map, no tonemap, 800x450).

## Increment 10

Same courtyard as increment 9 (bowl + rock + metal ball + copper pillar). Adds one warm point light beside the pillar; the directional stays. Our Cook-Torrance path adds the point light with inverse-square falloff so the pillar and nearby floor read a local highlight the sun alone would not make. IBL is unchanged.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment10.sh
```

Writes:

- `artifacts/increment10/scene.json` — authored scene (increment-9 set + point light)
- `artifacts/increment10/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment10/frame.png` — our renderer, 800x450, post-step poses (IBL + directional + point)
- `artifacts/increment10/threejs-frame.png` — stock Three.js, same poses (PointLight + directional, no IBL/tonemap)

### CLI

```bash
agent-rig increment10 --out artifacts/increment10
agent-rig sim artifacts/increment10/scene.json --out artifacts/increment10-sim
agent-rig render artifacts/increment10/scene.json --physics artifacts/increment10/physics.json --out artifacts/increment10/frame.png
```

`sim` / `render` load the point light via the shared scene loader.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment10-threejs.sh
```

Writes `artifacts/increment10/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional + PointLight at the scene position, shadows on the directional, no env map, no tonemap, 800x450).

## Scene format

JSON object with `camera`, `lights`, and `bodies`.

- `camera`: `position`, `look_at`, `fov_y_deg`
- `lights`: `type: "directional"` with `direction`, `color`, `intensity`; or `type: "point"` with `position`, `color`, `intensity`
- `bodies`: `id`, `shape` (`box` + full `size` xyz, `sphere` + `radius`, or `mesh` + `path` + `collider`), `position`, optional `rotation_wxyz` (default `[1,0,0,0]`), `mass` (0 = static), optional `linear_velocity`, `material` (`albedo`, `roughness`, `metallic`, optional `albedo_map` PNG path)
- mesh `path` (`.obj`, `.gltf`, or `.glb`) and `albedo_map` are relative to the repo root (or the scene file). `collider` is `convex_hull` or `trimesh`. If `albedo_map` is set, the sampled texel replaces albedo on that body. glTF load is POSITION + optional TEXCOORD_0 + indices, TRIANGLES only. If the primitive has `pbrMetallicRoughness` (`baseColorFactor` and/or `baseColorTexture`, plus optional metallic/roughness factors), that drives the look; scene JSON `material` is the fallback when the glTF has no material.

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

Increment 6: two distinct mesh files load (different paths / vertex counts); the scene has two mesh bodies; after stepping there is a contact involving a mesh and the dump records collider type for both; `increment6` writes scene + physics dump + our PNG.

Increment 7: the environment / ground body is a triangle mesh (not a box); after stepping there is a contact against that mesh and the dump records `trimesh`; `increment7` writes scene + physics dump + our PNG.

Increment 8: a glTF/GLB file loads (vertex/triangle counts > 0); the scene has a mesh body whose path ends in `.gltf` or `.glb`; after stepping there is a contact involving that body and the dump records its collider type; `increment8` writes scene + physics dump + our PNG.

Increment 9: the glTF has `pbrMetallicRoughness.baseColorFactor` (and/or `baseColorTexture`); the loaded mesh uses that factor, not only the scene-JSON albedo (JSON is a dull-gray fallback); after stepping there is a contact involving the glTF body and the dump records its collider type; `increment9` writes scene + physics dump + our PNG.

Increment 10: the scene has a point light (`position`, `color`, `intensity`) plus the existing directional; `sim` / `render` load it; after stepping the glTF pillar still has a collider type and a contact in the dump; `increment10` writes scene + physics dump + our PNG (local highlight / falloff the directional alone would not make).
