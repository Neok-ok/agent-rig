# agent-rig (increments 1–32)

Agent-native scene file + physics inspect + headless PNG. One command writes a JSON scene an agent can author, steps a real physics world, dumps body state and contacts, and renders the post-step frame with a small CPU Cook-Torrance raytracer plus procedural IBL (spheres, boxes, and triangle meshes from OBJ or a constrained glTF/GLB, with optional albedo textures and glTF pbrMetallicRoughness, including metallicRoughnessTexture, optional tangent-space normalTexture, optional emissiveFactor × emissiveTexture, optional alphaMode BLEND/MASK with non-1 alpha, optional occlusionTexture (AO, R channel), optional KHR_materials_transmission + IOR with Snell refraction, optional KHR_materials_volume Beer-Lambert attenuation, optional authored anisotropy + anisotropy_rotation on the gold ball, and optional authored iridescence + iridescence_ior + iridescence_thickness (thin-film rainbow) on the gold ball, and optional authored KHR_materials_dispersion on the glass pane so refracted highlights split into chromatic R/G/B fringes). Directional and point lights both cast shadow rays. A rectangular area light is multi-sampled for a soft penumbra. No GPU. No Three.js in the engine.

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


## Increment 11

Same courtyard as increment 10 (bowl + rock + metal ball + copper pillar, directional + warm point light). The point light now casts a shadow ray: from the hit toward the lamp, if anything sits closer than the lamp, that point-light contribution is skipped. The pillar’s local umbra on the bowl floor is the readable cue an unshadowed point light would not make. IBL is unchanged.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment11.sh
```

Writes:

- `artifacts/increment11/scene.json` — authored scene (increment-10 courtyard + both lights)
- `artifacts/increment11/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment11/frame.png` — our renderer, 800x450, post-step poses (IBL + directional + shadowed point)
- `artifacts/increment11/threejs-frame.png` — stock Three.js, same poses (PointLight + directional, both cast shadows, no IBL/tonemap)

### CLI

```bash
agent-rig increment11 --out artifacts/increment11
agent-rig sim artifacts/increment11/scene.json --out artifacts/increment11-sim
agent-rig render artifacts/increment11/scene.json --physics artifacts/increment11/physics.json --out artifacts/increment11/frame.png
```

`sim` / `render` load the point light via the shared scene loader.

### Three.js baseline (same post-step poses)

```bash
cd /workspace/agent-rig && ./scripts/increment11-threejs.sh
```

Writes `artifacts/increment11/threejs-frame.png` (stock MeshStandardMaterial, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450).


## Increment 12

Same courtyard as increment 11 (bowl + rock + metal ball + copper pillar, directional + shadowed point light). Physics is stepped once (same step count as increment 11) so the dump still has the pillar collider and ground–pillar contacts. Then eight cameras orbit the existing look-at at the increment-11 radius and height (yaw 0°, 45°, …, 315° around Y). Frozen pose — not a new physics animation.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment12.sh
```

Writes:

- `artifacts/increment12/scene.json` — authored increment-11 courtyard
- `artifacts/increment12/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment12/frame_00.png` … `frame_07.png` — our renderer, 800x450, same pose, 8 orbit cameras
- `artifacts/increment12/threejs_00.png` … `threejs_07.png` — stock Three.js, same 8 poses (no IBL/tonemap)

### CLI

```bash
agent-rig increment12 --out artifacts/increment12
```

### Three.js baseline (same 8 poses)

```bash
cd /workspace/agent-rig && ./scripts/increment12-threejs.sh
```

Writes `artifacts/increment12/threejs_00.png` … `threejs_07.png` (stock MeshStandardMaterial, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450). Camera position is passed as a `cam` query param.


## Increment 13

Same courtyard as increment 12 (bowl + rock + metal ball + copper pillar, directional + shadowed point light). The pillar glTF now has a packed `metallicRoughnessTexture` (G=roughness, B=metallic) with high-contrast horizontal bands — chrome stripes vs matte — so metal/roughness vary across the surface. Scene-JSON pillar material stays dull (metallic 0, roughness 0.85) and must not win over the texture. Single pose (no 8-orbit).

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment13.sh
```

Writes:

- `artifacts/increment13/scene.json` — authored increment-11 courtyard (dull JSON pillar fallback)
- `artifacts/increment13/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment13/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment13/threejs-frame.png` — stock Three.js, same pose (metalnessMap + roughnessMap from the glTF, no IBL/tonemap)

### CLI

```bash
agent-rig increment13 --out artifacts/increment13
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment13-threejs.sh
```

Writes `artifacts/increment13/threejs-frame.png` (stock MeshStandardMaterial, same packed MR map on metalnessMap and roughnessMap, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450).

## Increment 14

Same courtyard as increment 13 (bowl + rock + metal ball + copper pillar with metallicRoughnessTexture, directional + shadowed point light). The pillar glTF now also has a high-contrast tangent-space `normalTexture` (brick / corrugation, OpenGL +Z up). Scene-JSON has no normal map; bump lighting comes from the file. TBN is built from a TANGENT accessor if present, otherwise from triangle positions + TEXCOORD_0. Single pose.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment14.sh
```

Writes:

- `artifacts/increment14/scene.json` — authored increment-11 courtyard (no normal map in JSON)
- `artifacts/increment14/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment14/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment14/threejs-frame.png` — stock Three.js, same pose (normalMap from the glTF, no IBL/tonemap)

### CLI

```bash
agent-rig increment14 --out artifacts/increment14
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment14-threejs.sh
```

Writes `artifacts/increment14/threejs-frame.png` (stock MeshStandardMaterial, same glTF normalMap + MR maps, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450).

## Increment 15

Same courtyard as increment 14 (bowl + rock + metal ball + copper pillar with metallicRoughnessTexture + normalTexture, directional + shadowed point light). The ball is given a shove (non-zero linear velocity) and physics is stepped across 8 frames from the increment-11 courtyard camera. Opposite of increment 12: one camera, eight physics poses — no orbit.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment15.sh
```

Writes:

- `artifacts/increment15/scene.json` — increment-14 courtyard plus ball linear velocity
- `artifacts/increment15/physics.json` — trajectory dump (8 frames: positions, velocities, contacts; pillar collider)
- `artifacts/increment15/frame_00.png` … `frame_07.png` — our renderer, 800x450, same camera, 8 physics poses
- `artifacts/increment15/threejs_00.png` … `threejs_07.png` — stock Three.js, same 8 poses (no IBL/tonemap)

### CLI

```bash
agent-rig increment15 --out artifacts/increment15
```

`--frames` is the number of PNGs (default 8). The world is stepped `stride` times between frames (default 12).

### Three.js baseline (same 8 poses)

```bash
cd /workspace/agent-rig && ./scripts/increment15-threejs.sh
```

Writes `artifacts/increment15/threejs_00.png` … `threejs_07.png` (stock MeshStandardMaterial, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450). Loads the increment-15 scene JSON and applies each trajectory frame.


## Increment 16

Same courtyard as increment 14 (bowl + rock + metal ball + copper pillar with metallicRoughnessTexture + normalTexture, directional + shadowed point light). Single still — the increment-14 frozen pose, not another 8-frame sim. The pillar glTF now also has `emissiveFactor` and a high-contrast `emissiveTexture` (cyan glow bands / runes on black). Sampled emissive is `factor * textureRGB` and is added to outgoing radiance after lighting (self-illumination, not a new light). Scene-JSON has no emissive map.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment16.sh
```

Writes:

- `artifacts/increment16/scene.json` — increment-14 courtyard (no emissive in JSON)
- `artifacts/increment16/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment16/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment16/threejs-frame.png` — stock Three.js, same pose (emissive + emissiveMap from the glTF, no IBL/tonemap)

### CLI

```bash
agent-rig increment16 --out artifacts/increment16
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment16-threejs.sh
```

Writes `artifacts/increment16/threejs-frame.png` (stock MeshStandardMaterial, same glTF emissive / emissiveMap + MR + normalMap, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450).


## Increment 17

Same courtyard as increment 16 (bowl + rock + metal ball + copper pillar with metallicRoughnessTexture + normalTexture + emissive cyan bands, directional + shadowed point light). Single still. A new thin glass pane (`meshes/pane.gltf`) stands in front of the look: `alphaMode` BLEND, `baseColorFactor` `[0.75, 0.9, 1.0, 0.32]`. The raytracer shades the pane, continues the ray, and blends `src * alpha + behind * (1 - alpha)` (recurse depth 4). No refraction / IOR / transmission. The pillar stays opaque. Scene-JSON has no alpha map.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment17.sh
```

Writes:

- `artifacts/increment17/scene.json` — increment-16 courtyard plus the pane
- `artifacts/increment17/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment17/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment17/threejs-frame.png` — stock Three.js, same pose (`transparent` + opacity from the glTF, no IBL/tonemap)

### CLI

```bash
agent-rig increment17 --out artifacts/increment17
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment17-threejs.sh
```

Writes `artifacts/increment17/threejs-frame.png` (stock MeshStandardMaterial, pane `transparent` + opacity 0.32, same glTF emissive / MR / normalMap on the pillar, ambient 0.25 + directional + PointLight, both lights cast shadows, no env map, no tonemap, 800x450).


## Increment 18

Same courtyard as increment 17 (bowl + rock + metal ball + copper pillar with MR + normal + emissive, glass pane, directional). Single still. The warm lamp is a rectangular area light (`type: "area"`, position `[0.15, 1.45, 0.40]`, size `[1.2, 0.8]`, intensity `40`, normal facing down). The raytracer samples a 4×4 grid on that rectangle, traces a shadow ray per sample, and averages the unoccluded Cook-Torrance + 1/r² contribution. Larger authored size → softer penumbra (readable contact under the ball / pillar). No rewrite of IBL.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment18.sh
```

Writes:

- `artifacts/increment18/scene.json` — increment-17 courtyard; point lamp replaced by the area light
- `artifacts/increment18/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment18/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment18/threejs-frame.png` — stock Three.js, same pose (`RectAreaLight`, no IBL/tonemap; Three.js area lights do not cast a real penumbra)

### CLI

```bash
agent-rig increment18 --out artifacts/increment18
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment18-threejs.sh
```

Writes `artifacts/increment18/threejs-frame.png` (stock MeshStandardMaterial, same glTF materials, ambient 0.25 + directional + RectAreaLight at the authored pose, no env map, no tonemap, 800x450).


## Increment 19

Same courtyard as increment 18 (bowl + rock + metal ball + copper pillar with MR + normal + emissive, glass pane, directional + area light). Single still. The pillar glTF now has `occlusionTexture` (`textures/pillar-ao.png`, R = AO). Sampled AO multiplies IBL / ambient so flute grooves and the base contact band read darker than increment 18. Direct directional / area lighting is not multiplied by AO. Scene JSON is the increment-18 courtyard; the look lives on the glTF file.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment19.sh
```

Writes:

- `artifacts/increment19/scene.json` — increment-18 courtyard (AO is on `pillar.gltf`)
- `artifacts/increment19/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment19/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment19/threejs-frame.png` — stock Three.js, same pose (`MeshStandardMaterial.aoMap`, no env map, no tonemap)

### CLI

```bash
agent-rig increment19 --out artifacts/increment19
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment19-threejs.sh
```

Writes `artifacts/increment19/threejs-frame.png` (stock MeshStandardMaterial, same glTF materials including `aoMap` from `occlusionTexture`, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450).

## Increment 20

Same courtyard as increment 19 (bowl + rock + metal ball + copper pillar with MR + normal + emissive + AO, glass pane, directional + area light). Single still. The pane glTF now authors `KHR_materials_transmission` (`transmissionFactor` 1.0) and IOR 1.5 (`materials.ior` / `KHR_materials_ior`). The pane is a two-face slab (~0.48 mean thickness, 8° wedge so enter+exit do not cancel) yawed −15° about Y in increment 20's own scene JSON for a more oblique incidence. On a transmitting hit the ray refracts with Snell's law using the authored IOR (`eta = 1/ior` entering, `ior` leaving), nudges off the hit face, and continues so the bowl rim and ball are obviously kinked at the pane edge, not only tinted. Increment 17 was alpha composite only.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment20.sh
```

Writes:

- `artifacts/increment20/scene.json` — increment-19 courtyard with the pane yawed −15° about Y (transmission + IOR live on `pane.gltf`)
- `artifacts/increment20/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment20/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment20/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` transmission + ior, no env map, no tonemap)

### CLI

```bash
agent-rig increment20 --out artifacts/increment20
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment20-threejs.sh
```

Writes `artifacts/increment20/threejs-frame.png` (stock MeshPhysicalMaterial with `transmission` and `ior` from the glTF, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Ours still wins on IBL + real Snell vs stock.

## Increment 21

Same courtyard as increment 20 (bowl + rock + metal ball + copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR, directional + area light). Single still, same camera. Two new authored mesh bodies rest on the bowl floor: a wood crate (`meshes/crate.obj`, convex_hull) left-front of the rock, and a low bench (`meshes/bench.obj`, trimesh) right of the rock / in front of the pillar. Seven bodies total. No new light types, no clearcoat, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment21.sh
```

Writes:

- `artifacts/increment21/scene.json` — increment-20 courtyard plus crate and bench (own scene JSON)
- `artifacts/increment21/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment21/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment21/threejs-frame.png` — stock Three.js, same pose (no env map, no tonemap)

### CLI

```bash
agent-rig increment21 --out artifacts/increment21
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment21-threejs.sh
```

Writes `artifacts/increment21/threejs-frame.png` (stock MeshStandardMaterial / MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Ours still wins on IBL + refraction.

## Increment 22

Same courtyard as increment 21 (bowl + rock + metal ball + copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR, crate, bench, directional + area light). Single still, same camera. The gold ball authors a dielectric clearcoat layer (`clearcoat` + `clearcoat_roughness`) on top of the existing metallic-roughness Cook-Torrance — a wet / car-paint sheen, not a tweak of the base metal. Softness is the authored `clearcoat_roughness`. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment22.sh
```

Writes:

- `artifacts/increment22/scene.json` — increment-21 courtyard plus clearcoat on the gold ball (own scene JSON)
- `artifacts/increment22/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment22/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment22/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` clearcoat, no env map, no tonemap)

### CLI

```bash
agent-rig increment22 --out artifacts/increment22
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment22-threejs.sh
```

Writes `artifacts/increment22/threejs-frame.png` (stock MeshPhysicalMaterial with `clearcoat` + `clearcoatRoughness` from scene JSON, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Ours still wins on IBL sheen.


## Increment 23

Same courtyard as increment 22 (bowl + rock + metal ball with clearcoat, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR, crate, bench, directional + area light). Single still, same camera. The bench authors a fabric/velvet sheen layer (`sheen` + `sheen_roughness` + `sheen_color`) on top of the existing matte gray — a colored grazing Charlie lobe, not a base-albedo tint. Softness is the authored `sheen_roughness`; tint is the authored `sheen_color`. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment23.sh
```

Writes:

- `artifacts/increment23/scene.json` — increment-22 courtyard plus sheen on the bench (own scene JSON)
- `artifacts/increment23/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment23/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment23/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` sheen, no env map, no tonemap)

### CLI

```bash
agent-rig increment23 --out artifacts/increment23
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment23-threejs.sh
```

Writes `artifacts/increment23/threejs-frame.png` (stock MeshPhysicalMaterial with `sheen` + `sheenRoughness` + `sheenColor` from scene JSON, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Ours still wins on IBL sheen.

## Increment 24

Same courtyard as increment 23 (bowl + rock + metal ball with clearcoat, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR, crate, bench with sheen, directional + area light). Single still, same camera. The pane authors `KHR_materials_volume` (`attenuationColor` + `attenuationDistance` + `thicknessFactor`). Rays that travel through the glass pick up a Beer-Lambert color cast: `T = attenuationColor.pow(distance / attenuationDistance)` per channel, multiplied onto the continuing radiance. Longer path (thicker wedge) = stronger green. This is volume absorption, not a `baseColorFactor` / surface albedo tint. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment24.sh
```

Writes:

- `artifacts/increment24/scene.json` — increment-23 courtyard (own scene JSON; volume lives on `meshes/pane.gltf`)
- `artifacts/increment24/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment24/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment24/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` attenuationColor + attenuationDistance + thickness, no env map, no tonemap)

### CLI

```bash
agent-rig increment24 --out artifacts/increment24
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment24-threejs.sh
```

Writes `artifacts/increment24/threejs-frame.png` (stock MeshPhysicalMaterial with `attenuationColor` + `attenuationDistance` + `thickness` from the pane glTF, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450).

## Increment 25

Same courtyard as increment 24 (bowl + rock + metal ball with clearcoat, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR + volume attenuation, crate, bench with sheen, directional + area light). Single still, same camera. The gold ball authors `anisotropy` + `anisotropy_rotation` (keep the existing clearcoat). Specular uses an anisotropic GGX lobe (Burley/Kulla): tangent/bitangent from a stable up vector, rotated by the authored rotation around N; `at = roughness * (1 + anisotropy)`, `ab = roughness * (1 - anisotropy)`. The highlight stretches into a brushed-metal streak, obvious vs increment 24's circular coat highlight. Strength and direction come from the authored fields, not a hidden constant. Applied on lights and IBL spec. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment25.sh
```

Writes:

- `artifacts/increment25/scene.json` — increment-24 courtyard plus anisotropy on the gold ball (own scene JSON)
- `artifacts/increment25/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment25/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment25/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` anisotropy + anisotropyRotation, no env map, no tonemap)

### CLI

```bash
agent-rig increment25 --out artifacts/increment25
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment25-threejs.sh
```

Writes `artifacts/increment25/threejs-frame.png` (stock MeshPhysicalMaterial with `anisotropy` + `anisotropyRotation` from scene JSON, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450).

## Increment 26

Same courtyard as increment 25 (bowl + rock + metal ball with clearcoat + anisotropy, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR + volume attenuation, crate, bench with sheen, directional + area light). Single still, same camera. The gold ball authors `iridescence` + `iridescence_ior` + `iridescence_thickness` (keep the existing clearcoat + anisotropy). Specular F0 / Fresnel is tinted with a view-dependent thin-film hue from optical path `2 * n * d * cos(θ)` (Belcour/Barla compact equivalent). Factor, IOR, and thickness come from the authored fields, not a hidden constant. Extra layer on top of metal + clearcoat + anisotropy, not a base-albedo dye. Applied on lights and IBL spec so the brushed streak picks up a rainbow / oil-slick color shift, obvious vs increment 25's gold streak. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment26.sh
```

Writes:

- `artifacts/increment26/scene.json` — increment-25 courtyard plus iridescence on the gold ball (own scene JSON)
- `artifacts/increment26/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment26/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment26/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` iridescence + iridescenceIOR + iridescenceThicknessRange, no env map, no tonemap)

### CLI

```bash
agent-rig increment26 --out artifacts/increment26
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment26-threejs.sh
```

Writes `artifacts/increment26/threejs-frame.png` (stock MeshPhysicalMaterial with `iridescence` + `iridescenceIOR` + `iridescenceThicknessRange` from scene JSON, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450).

## Increment 27

Same courtyard as increment 26 (bowl + rock + metal ball with clearcoat + anisotropy + iridescence, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR + volume attenuation, crate, bench with sheen, directional + area light). Single still, same camera. The existing pane authors `KHR_materials_dispersion` (`dispersion` = 0.18). Refracted rays through the pane are split into separate R/G/B paths with Cauchy IOR `n(λ) = ior + dispersion * (1/λ² − 1/0.55²)` (λ in µm: R=0.65, G=0.55, B=0.45). Strength comes from the authored dispersion value, not a hidden constant. Zero dispersion keeps increment-26 single-ray refraction. Volume Beer-Lambert still applies per-ray (green tint stays). Chromatic fringes on the bowl rim / ball / pillar through the pane, obvious vs increment 26's monochromatic green volume tint. No new lights, no new bodies, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment27.sh
```

Writes:

- `artifacts/increment27/scene.json` — increment-26 courtyard plus dispersion on the pane (own scene JSON)
- `artifacts/increment27/physics.json` — post-step dump (poses, velocities, contacts, collider kinds)
- `artifacts/increment27/frame.png` — our renderer, 800x450, post-step pose
- `artifacts/increment27/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` dispersion, no env map, no tonemap)

### CLI

```bash
agent-rig increment27 --out artifacts/increment27
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment27-threejs.sh
```

Writes `artifacts/increment27/threejs-frame.png` (stock MeshPhysicalMaterial with `dispersion` from glTF/scene, plus existing transmission / ior / thickness / attenuationColor / attenuationDistance / clearcoat / sheen / anisotropy / iridescence, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450).

## Increment 28

Same courtyard as increment 27 (bowl + rock + gold ball with clearcoat 1.0 / roughness 0.08, anisotropy 0.95 / rotation 0.6, iridescence 1.0 / IOR 1.3 / thickness 380 nm, copper pillar with MR + normal + emissive + AO, glass pane with transmission + IOR 1.5 + volume attenuation + KHR_materials_dispersion 0.18, crate, bench with sheen 1.0 / roughness 0.4 / color [0.75, 0.12, 0.28], directional + area light). One hanging lantern (sphere, mass 0.4) is attached to the existing pillar by an authored Rapier hinge. Anchor is world-space (converted to local on each body); axis is world X so gravity swings it down. After stepping, the lantern hangs below the attachment — not a floating T-pose. Dump records the joint. No new material features, no new lights, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment28.sh
```

Writes:

- `artifacts/increment28/scene.json` — increment-27 courtyard plus lantern + hinge
- `artifacts/increment28/physics.json` — post-step dump (poses, contacts, collider kinds, joints)
- `artifacts/increment28/frame.png` — our renderer, 800x450, post-step hanging pose
- `artifacts/increment28/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment28 --out artifacts/increment28
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment28-threejs.sh
```

Writes `artifacts/increment28/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Joints are ignored by Three.js; the still uses our post-step dump poses.

## Increment 29

Same courtyard as increment 28 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge, directional + area light). One drawer (box, mass 0.3) is attached to the existing crate by an authored Rapier slider (`type: slider`, world-space `axis` toward the camera, `limits` `[0, 0.35]`). The drawer starts seated against the crate +Z face and is given an initial +Z velocity so it slides open. After stepping, the drawer COM has translated along the axis — not flush/nested as the start pose. Dump records both the slider and the existing hinge. No new material features, no new lights, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment29.sh
```

Writes:

- `artifacts/increment29/scene.json` — increment-28 courtyard plus drawer + slider
- `artifacts/increment29/physics.json` — post-step dump (poses, contacts, collider kinds, joints)
- `artifacts/increment29/frame.png` — our renderer, 800x450, post-step open pose
- `artifacts/increment29/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment29 --out artifacts/increment29
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment29-threejs.sh
```

Writes `artifacts/increment29/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Joints are ignored by Three.js; the still uses our post-step dump poses.


## Increment 30

Same courtyard as increment 29 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge, drawer + slider, directional + area light). The existing pillar–lantern hinge authors a Rapier revolute motor (`motor_target_velocity` 4 rad/s, `motor_max_force` 8) so the lantern is driven around the hinge axis instead of damping to a hang. No new joint type, no new body. After stepping, the lantern is swung aside vs increment 29's hang-down pose. Dump records motor fields on the hinge plus the existing slider. No new material features, no new lights, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment30.sh
```

Writes:

- `artifacts/increment30/scene.json` — increment-29 courtyard plus authored hinge motor
- `artifacts/increment30/physics.json` — post-step dump (poses, contacts, collider kinds, joints including motor fields)
- `artifacts/increment30/frame.png` — our renderer, 800x450, post-step swung pose
- `artifacts/increment30/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment30 --out artifacts/increment30
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment30-threejs.sh
```

Writes `artifacts/increment30/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Joints are ignored by Three.js; the still uses our post-step dump poses.


## Increment 31

Same courtyard as increment 30 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor, drawer + slider, directional + area light). Adds one small metal charm hanging from the lantern on an authored Rapier spherical / ball socket (`type: ball`, world-space `anchor`). After stepping, the charm hangs below the socket (not a T-pose) and is free in 2 axes (not locked to the lantern hinge swing plane). Dump records the ball (kind, lantern, charm, anchor) plus the existing hinge motor and slider. No new material features, no new lights, no IBL rewrite.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment31.sh
```

Writes:

- `artifacts/increment31/scene.json` — increment-30 courtyard plus charm + ball socket
- `artifacts/increment31/physics.json` — post-step dump (poses, contacts, collider kinds, joints including ball + hinge motor + slider)
- `artifacts/increment31/frame.png` — our renderer, 800x450, post-step hanging pose
- `artifacts/increment31/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment31 --out artifacts/increment31
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment31-threejs.sh
```

Writes `artifacts/increment31/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Joints are ignored by Three.js; the still uses our post-step dump poses.

## Increment 32

Same courtyard as increment 31 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor, drawer + slider, charm + ball socket, directional + area light). Adds one authorable trigger / sensor volume at the open-drawer pose (`id: "drawer_open"`, box size `[0.30, 0.22, 0.28]`, position `[-0.35, 0.10, 1.37]`). Mapped to a Rapier sensor collider (`sensor = true`, no contact forces). After stepping, the open drawer overlaps the trigger; the dump records `overlaps: [{ "trigger": "drawer_open", "body": "drawer" }]` plus the existing joints (ball + hinge motor + slider). The trigger renders as a translucent cyan box (BLEND, alpha ~0.18). No new joints, no new lights, no IBL rewrite, no new material features on existing bodies. `increment31_scene()` stays trigger-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment32.sh
```

Writes:

- `artifacts/increment32/scene.json` — increment-31 courtyard plus drawer-open trigger
- `artifacts/increment32/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps)
- `artifacts/increment32/frame.png` — our renderer, 800x450, post-step pose with cyan sensor box
- `artifacts/increment32/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment32 --out artifacts/increment32
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment32-threejs.sh
```

Writes `artifacts/increment32/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, no env map, no tonemap, 800x450). Triggers draw as transparent cyan boxes; bodies still use our post-step dump poses.

## Scene format

JSON object with `camera`, `lights`, `bodies`, optional `joints`, and optional `triggers`.

- `camera`: `position`, `look_at`, `fov_y_deg`
- `lights`: `type: "directional"` with `direction`, `color`, `intensity`; or `type: "point"` with `position`, `color`, `intensity`; or `type: "area"` with `position`, `size` `[width, height]`, `color`, `intensity`, optional `normal` (default `[0,-1,0]`). Area softness comes from the authored `size`.
- `bodies`: `id`, `shape` (`box` + full `size` xyz, `sphere` + `radius`, or `mesh` + `path` + `collider`), `position`, optional `rotation_wxyz` (default `[1,0,0,0]`), `mass` (0 = static), optional `linear_velocity`, `material` (`albedo`, `roughness`, `metallic`, optional `albedo_map` PNG path, optional `clearcoat` + `clearcoat_roughness`, optional `sheen` + `sheen_roughness` + `sheen_color`, optional `anisotropy` + `anisotropy_rotation`, optional `iridescence` + `iridescence_ior` + `iridescence_thickness`, optional `dispersion`)
- `triggers` (optional, default empty): `{ "id", "shape": { "type": "box", "size": [x,y,z] }, "position": [x,y,z] }`. Mapped to a Rapier sensor cuboid (`sensor = true`) on a fixed body. Sensors report overlaps and do not generate contact forces or push bodies. After stepping, the dump records `overlaps: [{ "trigger", "body" }]`. Rendered as a translucent cyan box (BLEND, alpha ~0.18).
- `joints` (optional, default empty): `{ "type": "hinge", "body_a", "body_b", "anchor": [x,y,z], "axis": [x,y,z] }` with optional `motor_target_velocity` + `motor_max_force` (serde default 0; both 0 = hang damper), or `{ "type": "slider", "body_a", "body_b", "axis": [x,y,z], "limits": [min, max] }` (optional `anchor` is the closed-pose world attachment), or `{ "type": "ball", "body_a", "body_b", "anchor": [x,y,z] }`. `anchor` is world-space (converted to local on each body at spawn). `axis` is a world-space direction; hinge axes should be horizontal so gravity hangs the child. Mapped to a Rapier revolute, prismatic, or spherical impulse joint. A nonzero hinge motor replaces the hang damper with `motor_velocity(target, max_force)`. A ball socket is free in 2 axes (not locked to a hinge swing plane).
- mesh `path` (`.obj`, `.gltf`, or `.glb`) and `albedo_map` are relative to the repo root (or the scene file). `collider` is `convex_hull` or `trimesh`. If `albedo_map` is set, the sampled texel replaces albedo on that body. glTF load is POSITION + optional TEXCOORD_0 + indices, TRIANGLES only. If the primitive has `pbrMetallicRoughness` (`baseColorFactor` and/or `baseColorTexture`, plus optional metallic/roughness factors and optional `metallicRoughnessTexture`), that drives the look; scene JSON `material` is the fallback when the glTF has no material. When `metallicRoughnessTexture` is present, roughness is sampled from G and metallic from B (times the factors); scene-JSON metallic/roughness do not override the texel. When `normalTexture` is present, RGB is unpacked to a tangent-space normal (2c−1) and TBN·n_ts replaces the geometric N for lighting (TANGENT accessor, or TBN derived from triangle positions + TEXCOORD_0). Scene JSON has no normal map. When `emissiveFactor` and/or `emissiveTexture` are present, sampled emissive is `factor * textureRGB` (no texture → factor only) and is added to outgoing radiance after lighting. Scene JSON has no emissive map. When `alphaMode` is `BLEND`, the surface is shaded then the ray continues and the results are blended (`src * alpha + behind * (1-alpha)`); `MASK` discards below `alphaCutoff`. Alpha is `baseColorFactor[3]` times baseColorTexture A if present. When `KHR_materials_transmission.transmissionFactor` is > 0, the continuation uses Snell refraction with the authored IOR (`materials.ior` or `KHR_materials_ior`, glass ~1.5): `eta = 1/ior` entering, `ior` leaving. No hidden bend constant. When `KHR_materials_volume` is present, the enter→exit path through the volume applies Beer-Lambert: `T = attenuationColor.pow(distance / attenuationDistance)` per channel, multiplied onto the radiance behind the pane. `attenuationColor` and `attenuationDistance` are authored (not hidden constants); this is volume absorption, not a change to `baseColorFactor`. When `occlusionTexture` is present, the R channel is sampled as AO (0 = occluded, 1 = open) and multiplies IBL / ambient; directional and area direct lighting are not multiplied by AO. Scene JSON has no AO map. Optional `anisotropy` (0–1) and `anisotropy_rotation` (radians) on a body material stretch the specular into a brushed-metal GGX lobe; strength and direction are the authored values (default 0 = isotropic). Optional `iridescence` (0–1), `iridescence_ior` (default 1.3), and `iridescence_thickness` (nm, default 400) add a thin-film rainbow on the specular F0 / Fresnel; factor, IOR, and thickness are the authored values (default 0 = off). Optional `dispersion` (KHR_materials_dispersion, default 0) on a transmitting material splits refracted rays into R/G/B with Cauchy IOR `n(λ) = ior + dispersion * (1/λ² − 1/0.55²)`; strength is the authored value (0 = increment-26 single-ray refraction).

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

Increment 11: the point light casts a shadow ray (occluded if something sits between the hit and the lamp); the courtyard and both lights stay; after stepping the glTF pillar still has a collider type and a contact in the dump; `increment11` writes scene + physics dump + our PNG (local umbra the unshadowed point light would not make).

Increment 12: same courtyard and lights; physics is stepped once; eight orbit cameras write `frame_00.png`…`frame_07.png` that are not copies of one still; the dump still records the pillar collider and a contact involving it; `increment12` writes scene + physics dump + the 8 PNGs.

Increment 13: the glTF has `pbrMetallicRoughness.metallicRoughnessTexture`; the loaded mesh samples G/B (not the scene-JSON metallic 0 / roughness 0.85); after stepping there is a contact involving the glTF body and the dump records its collider type; `increment13` writes scene + physics dump + our PNG.

Increment 14: the glTF has `normalTexture`; the loaded mesh shades with sampled tangent-space normals (TBN from TANGENT or from triangle + UV), not the geometric N alone; after stepping there is a contact involving the glTF body and the dump records its collider type; `increment14` writes scene + physics dump + our PNG.

Increment 15: same courtyard and lights; the ball has a non-zero initial linear velocity; physics is stepped across 8 frames from the increment-11 camera (no orbit); `frame_00` and `frame_07` are not copies; after the sim the dump still records the pillar collider and a contact involving it; `increment15` writes scene + physics dump + the 8 PNGs.

Increment 16: the glTF has `emissiveTexture` (and `emissiveFactor`); the loaded mesh samples `factor * textureRGB` (non-zero, not the no-texture fallback); after stepping there is a contact involving the glTF body and the dump records its collider type; `increment16` writes scene + physics dump + our PNG.

Increment 17: a glTF has `alphaMode` BLEND (or MASK) and sampled alpha is not 1.0 everywhere; the courtyard (bowl + rock + ball + pillar + pane) stays; after stepping there is a contact involving a glTF body and the dump records a collider type; `increment17` writes scene + physics dump + our PNG.

Increment 18: the scene has an area light (`position`, `size`, `color`, `intensity`); the courtyard including the pane stays; after stepping there is a contact involving a glTF body and the dump records a collider type; `increment18` writes scene + physics dump + our PNG (soft penumbra, not a hard point-light umbra).

Increment 19: the glTF has `occlusionTexture`; sampled AO is not 1.0 everywhere; the courtyard including the pane and area light stays; after stepping there is a contact involving a glTF body and the dump records a collider type; `increment19` writes scene + physics dump + our PNG (crevices / contact band darker than increment 18).

Increment 20: the pane glTF has transmission and IOR (IOR is not 1.0; transmission > 0 and IOR > 1); the courtyard including the pane, area light, and AO pillar stays; after stepping there is a contact involving a glTF body and the dump records a collider type; `increment20` writes scene + physics dump + our PNG (bowl rim / ball bent through the pane).

Increment 21: scene has ≥7 bodies; courtyard including pane + area light + AO pillar stays; crate (convex_hull) and bench (trimesh) are authored meshes that rest on the bowl; after stepping there is a contact involving a glTF body and the dump records its collider type; new bodies appear in the dump with collider kinds; `increment21` writes scene + physics dump + our PNG.

Increment 22: scene has clearcoat + clearcoat_roughness on the gold ball; courtyard including crate + bench + pane + area light + AO pillar stays; after stepping there is a contact involving a glTF body and the dump records its collider type; `increment22` writes scene + physics dump + our PNG.

Increment 23: scene has sheen + sheen_color on the bench; courtyard including crate + pane + area light + AO pillar + clearcoat ball stays; after stepping there is a contact involving a glTF body and the dump records its collider type; `increment23` writes scene + physics dump + our PNG.

Increment 24: pane glTF has `KHR_materials_volume` (`attenuationColor` not white + authored `attenuationDistance`); courtyard including crate + sheen bench + area light + AO pillar + clearcoat ball stays; after stepping there is a contact involving a glTF body and the dump records its collider type; `increment24` writes scene + physics dump + our PNG.

Increment 25: scene has anisotropy + anisotropy_rotation on the gold ball; courtyard including crate + sheen bench + volume pane + area light + AO pillar + clearcoat stays; after stepping there is a contact involving a glTF body and the dump records its collider type; `increment25` writes scene + physics dump + our PNG.

Increment 26: scene has iridescence + iridescence_ior + iridescence_thickness on the gold ball; courtyard including crate + sheen bench + volume pane + area light + AO pillar + clearcoat + anisotropy stays; after stepping there is a contact involving a glTF body and the dump records its collider type; `increment26` writes scene + physics dump + our PNG.

Increment 27: pane glTF has `KHR_materials_dispersion` (authored `dispersion` > 0); courtyard including crate + sheen bench + volume pane + IOR + area light + AO pillar + clearcoat + anisotropy + iridescence stays; after stepping there is a contact involving a glTF body and the dump records its collider type (pillar `convex_hull` + ground–pillar); `increment27` writes scene + physics dump + our PNG.

Increment 28: courtyard plus one hanging lantern (sphere) on the existing copper pillar via an authored Rapier hinge (`type: hinge`, world-space `anchor` + horizontal `axis`); after stepping the lantern hangs below the hinge (not a floating T-pose); dump records the joint (kind, bodies, axis/anchor) plus pillar `convex_hull` and ground–pillar contacts; courtyard (crate, sheen bench, volume+dispersion pane, area light, AO pillar, clearcoat+anisotropy+iridescence ball) stays; `increment28` writes scene + physics dump + our PNG.

Increment 29: courtyard plus one drawer (box) on the existing crate via an authored Rapier slider (`type: slider`, world-space `axis` + `limits`); after stepping the drawer COM has translated along the axis (open, not flush); dump records the slider (kind, crate, drawer, axis, limits) and the existing hinge (pillar, lantern) plus pillar `convex_hull` and ground–pillar contacts; courtyard (lantern + hinge, crate, sheen bench, volume+dispersion pane, area light, AO pillar, clearcoat+anisotropy+iridescence ball) stays; `increment29` writes scene + physics dump + our PNG.

Increment 30: same courtyard as increment 29; the existing pillar–lantern hinge authors `motor_target_velocity` ~4 rad/s and `motor_max_force` ~8 (no new joint type, no new body); after stepping the lantern is swung aside vs increment 29's hang-down pose (COM not ~0.32 below the hinge on the same hang line); dump records motor fields on the hinge plus the existing slider; courtyard (drawer + slider, lantern + hinge, crate, sheen bench, volume+dispersion pane, area light, AO pillar, clearcoat+anisotropy+iridescence ball) stays; `increment30` writes scene + physics dump + our PNG.

Increment 31: courtyard plus one charm (sphere) hanging from the lantern via an authored Rapier ball socket (`type: ball`, world-space `anchor`); after stepping the charm COM hangs below the socket (not a T-pose) and keeps 2-axis freedom off the hinge swing plane; dump records the ball (kind, lantern, charm, anchor) AND the hinge with motor fields AND the slider; pillar `convex_hull` + ground–pillar contacts stay; courtyard (hinge motor 4/8, drawer + slider, crate, sheen bench, volume+dispersion pane, area light, AO pillar, clearcoat+anisotropy+iridescence ball) stays; `increment31` writes scene + physics dump + our PNG.

Increment 32: courtyard plus one authorable trigger / sensor volume at the open-drawer pose (`id: drawer_open`, box size `[0.30, 0.22, 0.28]`, position `[-0.35, 0.10, 1.37]`); after stepping the open drawer overlaps the trigger; dump records `overlaps: [{ trigger: drawer_open, body: drawer }]` AND the ball + hinge motor + slider; pillar `convex_hull` + ground–pillar contacts stay; courtyard (charm + ball, hinge motor 4/8, drawer + slider, crate, sheen bench, volume+dispersion pane, area light, AO pillar, clearcoat+anisotropy+iridescence ball) stays; increment 1–31 scene JSON and `increment31_scene()` stay trigger-free; `increment32` writes scene + physics dump + our PNG.
