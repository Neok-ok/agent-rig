# agent-rig (increments 1–54)

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

## Increment 33

Same courtyard as increment 32 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor, drawer + slider, charm + ball socket, drawer-open trigger, directional + area light). The existing lantern authors `emissive: [1.0, 0.55, 0.12]` and `emissive_intensity: 16` so it self-glows and lights nearby surfaces (pillar / ground / charm get a warm wash). The lantern COM is treated as a spherical / point mesh light; it is not a new `lights[]` entry, not a new body, and not a new joint. `increment32_scene()` stays dark (intensity 0). Three.js can glow the lantern (`MeshPhysicalMaterial.emissive`) but does not wash the courtyard from it.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment33.sh
```

Writes:

- `artifacts/increment33/scene.json` — increment-32 courtyard plus lantern emissive / intensity
- `artifacts/increment33/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps)
- `artifacts/increment33/frame.png` — our renderer, 800x450, glowing lantern + courtyard wash
- `artifacts/increment33/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial` emissive, no env map, no tonemap)

### CLI

```bash
agent-rig increment33 --out artifacts/increment33
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment33-threejs.sh
```

Writes `artifacts/increment33/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js glows the lantern but does not mesh-light the courtyard; bodies still use our post-step dump poses.

## Increment 34

Same courtyard as increment 33 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor, drawer + slider, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light). The existing crate–drawer slider authors `motor_target_velocity: -2.0` (toward closed / −Z) and `motor_max_force: 6.0`. Increment 34 zeros the drawer initial +Z velocity so the motor cleanly drives closed; increment 33 keeps `[0, 0, 2.5]` and opens to ~1.375. After stepping, drawer COM sits near z=1.02 (closed), not increment 33's open pose. The `drawer_open` trigger stays authored; overlap may be empty because the drawer left the open sensor. No new bodies, no new joint types, no new lights, no IBL rewrite. `increment33_scene()` does not grow slider motor fields.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment34.sh
```

Writes:

- `artifacts/increment34/scene.json` — increment-33 courtyard plus slider motor
- `artifacts/increment34/physics.json` — post-step dump (poses, contacts, collider kinds, joints including slider motor −2/6, overlaps)
- `artifacts/increment34/frame.png` — our renderer, 800x450, closed drawer seated on the crate
- `artifacts/increment34/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap)

### CLI

```bash
agent-rig increment34 --out artifacts/increment34
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment34-threejs.sh
```

Writes `artifacts/increment34/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Joints are ignored by Three.js; bodies still use our post-step dump poses (closed drawer).

## Increment 35

Same courtyard as increment 34 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light). The scene authors one physics ray `drawer_probe` from near `[-0.35, 0.55, 1.35]` toward the seated drawer COM `[-0.35, 0.10, 1.02]` (`max_toi` ~2). After stepping, the dump records a hit on the closed drawer (sensors/triggers are skipped so they do not steal the hit). Misses are omitted. The renderer draws a thin magenta segment from origin to hit plus a yellow hit marker. Three.js draws the same debug lines from the dump; it does not perform the physics raycast. No new bodies, joints, or lights. `increment34_scene()` stays raycast-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment35.sh
```

Writes:

- `artifacts/increment35/scene.json` — increment-34 courtyard plus `drawer_probe`
- `artifacts/increment35/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps, `ray_hits`)
- `artifacts/increment35/frame.png` — our renderer, 800x450, courtyard plus the probe segment hitting the closed drawer
- `artifacts/increment35/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); debug line from dump points

### CLI

```bash
agent-rig increment35 --out artifacts/increment35
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment35-threejs.sh
```

Writes `artifacts/increment35/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js draws the authored ray from dump `ray_hits`; it does not raycast. Bodies still use our post-step dump poses (closed drawer).

## Increment 36

Same courtyard as increment 35 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast). The scene authors one physics shapecast `drawer_sweep` from near `[-0.35, 0.55, 1.02]` downward `[0, -1, 0]` with a box of size `~[0.10, 0.04, 0.10]` (`max_toi` ~1). After stepping, the dump records a sweep hit on the closed drawer (sensors/triggers are skipped so they do not steal the hit). Misses are omitted. The increment-35 raycast path stays: `drawer_probe` still hits the drawer and the dump still has `ray_hits`. The renderer draws a translucent box at the hit pose (`origin + dir * toi`) plus a small hit marker (on a miss, the box is drawn at `origin + dir * max_toi`). Three.js draws the same debug sweep from dump `sweep_hits`; it does not perform the physics shapecast. No new bodies, joints, or lights. `increment35_scene()` stays shapecast-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment36.sh
```

Writes:

- `artifacts/increment36/scene.json` — increment-35 courtyard plus `drawer_sweep`
- `artifacts/increment36/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps, `ray_hits`, `sweep_hits`)
- `artifacts/increment36/frame.png` — our renderer, 800x450, courtyard plus the sweep box hitting the closed drawer
- `artifacts/increment36/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); debug sweep from dump points

### CLI

```bash
agent-rig increment36 --out artifacts/increment36
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment36-threejs.sh
```

Writes `artifacts/increment36/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js draws the authored sweep from dump `sweep_hits` and keeps the increment-35 ray debug lines; it does not shapecast. Bodies still use our post-step dump poses (closed drawer).

## Increment 37

Same courtyard as increment 36 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast). The scene authors one crate lid (box `~[0.28, 0.04, 0.28]` at `[-0.35, 0.28, 0.85]`, mass `~0.2`, wood albedo) welded to the crate with a Rapier fixed / weld joint (`type: "fixed"`, world-space `anchor` at the crate–lid interface `[-0.35, 0.26, 0.85]`). After stepping, the lid COM stays on the crate (y near authored `~0.28`, not on the ground). The dump records the fixed joint (`kind: "fixed"`, crate, lid) AND still has hinge / slider / ball + `ray_hits` drawer_probe + `sweep_hits` drawer_sweep. Three.js ignores joints; the lid uses the dump pose. No new lights. `increment36_scene()` stays lid-free and fixed-joint-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment37.sh
```

Writes:

- `artifacts/increment37/scene.json` — increment-36 courtyard plus lid + fixed joint
- `artifacts/increment37/physics.json` — post-step dump (poses, contacts, collider kinds, joints including `kind: "fixed"`, overlaps, `ray_hits`, `sweep_hits`)
- `artifacts/increment37/frame.png` — our renderer, 800x450, courtyard plus the seated lid
- `artifacts/increment37/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); lid from dump pose

### CLI

```bash
agent-rig increment37 --out artifacts/increment37
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment37-threejs.sh
```

Writes `artifacts/increment37/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores joints; bodies still use our post-step dump poses (seated lid, closed drawer).

## Increment 38

Same courtyard as increment 37 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint). The scene authors one impulse on the gold ball (`body: "ball"`, `linear: [1.8, 0.4, 0.5]`). Applied once at spawn via Rapier `apply_impulse` in world space at the center of mass (no `point`). After 120 steps the ball COM has rolled off its increment-37 seat at `x = -1.1` (`|ball.x + 1.1| > 0.25`). The dump records the authored impulse (not post-step velocity) AND still has lid + fixed + hinge / slider / ball + `ray_hits` drawer_probe + `sweep_hits` drawer_sweep. Three.js ignores impulses; the ball uses the dump pose. No new bodies, joints, or lights. `increment37_scene()` stays impulse-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment38.sh
```

Writes:

- `artifacts/increment38/scene.json` — increment-37 courtyard plus the ball impulse
- `artifacts/increment38/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`)
- `artifacts/increment38/frame.png` — our renderer, 800x450, courtyard plus the ball off its seat
- `artifacts/increment38/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); ball from dump pose

### CLI

```bash
agent-rig increment38 --out artifacts/increment38
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment38-threejs.sh
```

Writes `artifacts/increment38/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores impulses; bodies still use our post-step dump poses (ball off its seat, seated lid, closed drawer).

## Increment 39

Same courtyard as increment 38 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`). The scene authors one kinematic `platform` (box size `[0.55, 0.06, 0.35]`, position `[-0.55, 0.04, -0.55]`, `kinematic: true`, `linear_velocity: [0.45, 0.0, 0.0]`, slate albedo `[0.38, 0.40, 0.44]`, roughness `0.55`). Spawned as Rapier `KinematicVelocityBased`; each physics step re-applies the authored linear velocity (constant authored velocity, not a one-shot). After 120 steps the platform has slid +X (`|platform.x - authored_x| > 0.4`; `0.45 * 2s ≈ 0.9`). The dump records the platform pose AND `kinematic: true` on that body. Three.js ignores kinematic; the platform uses the dump pose. No new lights. Camera stays increment-38. `increment38_scene()` stays platform-free and kinematic-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment39.sh
```

Writes:

- `artifacts/increment39/scene.json` — increment-38 courtyard plus the kinematic platform
- `artifacts/increment39/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`)
- `artifacts/increment39/frame.png` — our renderer, 800x450, courtyard plus the sliding platform
- `artifacts/increment39/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); platform from dump pose

### CLI

```bash
agent-rig increment39 --out artifacts/increment39
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment39-threejs.sh
```

Writes `artifacts/increment39/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores kinematic; bodies still use our post-step dump poses (slid platform, ball off its seat, seated lid, closed drawer).


## Increment 40

Same courtyard as increment 39 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`). The scene authors one dynamic `rider` (box size `[0.16, 0.16, 0.16]`, position `[-0.55, 0.15, -0.55]` seated on the platform top, mass `0.35`, clay/terracotta albedo `[0.72, 0.38, 0.22]`, roughness `0.7`, not kinematic). After 120 steps the rider has ridden with the platform (`|rider.x - authored_x| > 0.35`) and its COM stays on the slab (not on the ground). The dump records the rider pose (dynamic) AND the platform pose with `kinematic: true`. Three.js uses dump poses for both. No new lights. Camera stays increment-39. `increment39_scene()` stays rider-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment40.sh
```

Writes:

- `artifacts/increment40/scene.json` — increment-39 courtyard plus the clay rider
- `artifacts/increment40/physics.json` — post-step dump (poses, contacts, collider kinds, joints, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, rider on the slab)
- `artifacts/increment40/frame.png` — our renderer, 800x450, courtyard plus the rider on the sliding platform
- `artifacts/increment40/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); rider + platform from dump poses

### CLI

```bash
agent-rig increment40 --out artifacts/increment40
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment40-threejs.sh
```

Writes `artifacts/increment40/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores kinematic; bodies still use our post-step dump poses (ridden rider, slid platform, ball off its seat, seated lid, closed drawer).


## Increment 41

Same courtyard as increment 40 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`). The scene authors one dynamic teal `gate` (box size `[0.06, 0.72, 0.42]`, position `[0.35, 0.40, 1.75]` in the camera-facing foreground, mass `0.5`, albedo `[0.18, 0.42, 0.38]`, roughness `0.45`, not kinematic) on a `ground`–`gate` Rapier hinge (`axis` `[0, 1, 0]`, world `anchor` `[0.35, 0.04, 1.75]`, `limits` `[0.0, 1.15]`, `motor_target_velocity` `1.4`, `motor_max_force` `5.0`). The motor drives the gate open; the authored limits stop it. After 120 steps the gate has yawed vs the authored pose and the hinge angle sits at/near the upper limit (within 0.2 rad of 1.15, not past 1.30). Dump records `limits` (and the current angle) on that hinge. The pillar–lantern hinge stays limit-free. Three.js ignores joints/limits and uses the dump gate pose. No new lights. Camera stays increment-40. `increment40_scene()` stays gate-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment41.sh
```

Writes:

- `artifacts/increment41/scene.json` — increment-40 courtyard plus the teal gate + limited hinge
- `artifacts/increment41/physics.json` — post-step dump (poses, contacts, collider kinds, joints including gate hinge `limits` + angle, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, rider on the slab)
- `artifacts/increment41/frame.png` — our renderer, 800x450, courtyard plus the yawed teal gate in the foreground
- `artifacts/increment41/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); gate from dump pose

### CLI

```bash
agent-rig increment41 --out artifacts/increment41
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment41-threejs.sh
```

Writes `artifacts/increment41/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores joints/limits; bodies still use our post-step dump poses (yawed gate, ridden rider, slid platform, ball off its seat, seated lid, closed drawer).


## Increment 42

Same courtyard as increment 41 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]`). The scene authors one dynamic brass `bob` (sphere radius `0.08`, position `[0.35, 0.88, 1.75]` above the gate top, mass `0.2`, albedo `[0.82, 0.64, 0.22]`, roughness `0.28`, metallic `0.85`, not kinematic) hung from the gate by a `gate`–`bob` Rapier rope (`type: "distance"`, world `anchor` `[0.35, 0.76, 1.75]`, `rest_length` `0.38`). Gravity drops the bob until the rope tautens. After 120 steps the bob has dropped below authored `0.88` by more than `0.12` and the distance from the current gate-top (local anchors track the yawing gate) to the bob COM is `<= rest_length + 0.08`. Dump records `kind: "distance"` plus `rest_length`. Three.js ignores the rope and uses the dump bob pose. No new lights. Camera stays increment-41. `increment41_scene()` stays bob-free and distance-joint-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment42.sh
```

Writes:

- `artifacts/increment42/scene.json` — increment-41 courtyard plus the brass bob + distance / rope joint
- `artifacts/increment42/physics.json` — post-step dump (poses, contacts, collider kinds, joints including distance `rest_length`, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, dropped bob on the rope)
- `artifacts/increment42/frame.png` — our renderer, 800x450, courtyard plus the hung brass bob in the foreground with the gate
- `artifacts/increment42/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); bob from dump pose

### CLI

```bash
agent-rig increment42 --out artifacts/increment42
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment42-threejs.sh
```

Writes `artifacts/increment42/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores the rope; bodies still use our post-step dump poses (hung bob, yawed gate, ridden rider, slid platform, ball off its seat, seated lid, closed drawer).


## Increment 43

Same courtyard as increment 42 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]`, brass `bob` on a `gate`–`bob` rope). The existing distance joint authors `break_force` ~1.5, and the scene appends one extra impulse on the bob (`linear` `[0.0, -4.0, 1.6]`). Each step, if the rope reaction magnitude exceeds `break_force`, the impulse joint is removed. After 120 steps the bob has fallen onto the bowl (`y < 0.22`, not hanging at ~0.38). The dump omits the broken gate–bob distance from `joints` and records `broken_joints: [{ "kind": "distance", "body_a": "gate", "body_b": "bob" }]`. `increment42_scene()` stays unbreakable (`break_force` 0 / omitted) and does not add the bob impulse — increment 42's bob still hangs. Three.js ignores the rope and uses the dump bob pose. No new lights. No new body. Camera stays increment-42.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment43.sh
```

Writes:

- `artifacts/increment43/scene.json` — increment-42 courtyard plus `break_force` on the gate–bob rope and the extra bob impulse
- `artifacts/increment43/physics.json` — post-step dump (poses, contacts, collider kinds, remaining joints, `broken_joints`, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, fallen bob)
- `artifacts/increment43/frame.png` — our renderer, 800x450, courtyard plus the fallen brass bob in the foreground with the gate
- `artifacts/increment43/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); bob from dump pose

### CLI

```bash
agent-rig increment43 --out artifacts/increment43
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment43-threejs.sh
```

Writes `artifacts/increment43/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores the rope; bodies still use our post-step dump poses (fallen bob, yawed gate, ridden rider, slid platform, ball off its seat, seated lid, closed drawer).

## Increment 44

Same courtyard as increment 43 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]`, brass `bob` on a breakable `gate`–`bob` rope). The scene authors one dynamic tan `cork` (sphere radius `0.14`, position `[0.35, 1.15, 1.75]` above the gate top, mass `0.25`, albedo `[0.72, 0.58, 0.32]`, roughness `0.8`, not kinematic) hung from the gate on a Rapier spring (`type: "spring"`, world `anchor` `[0.35, 0.76, 1.75]`, `rest_length` `0.42`, `stiffness` `40`, `damping` `4`). After 120 steps the cork has dropped from authored `1.15` by more than `0.15` and the distance from the current world-space gate-top to the cork COM is within `0.12` of rest length (settled, not flying away). Dump records `kind: "spring"` plus `rest_length`, `stiffness`, and `damping`. The fallen bob, broken rope, gate limits, rider, and platform stay. Three.js ignores the spring and uses the dump cork pose. No new lights. Camera stays increment-43. `increment43_scene()` stays cork-free and spring-free.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment44.sh
```

Writes:

- `artifacts/increment44/scene.json` — increment-43 courtyard plus the tan cork + gate–cork spring
- `artifacts/increment44/physics.json` — post-step dump (poses, contacts, collider kinds, joints including `kind: "spring"` + rest_length / stiffness / damping, `broken_joints`, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, fallen bob, settled cork)
- `artifacts/increment44/frame.png` — our renderer, 800x450, courtyard plus the cork on the gate and the fallen brass bob
- `artifacts/increment44/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); cork from dump pose

### CLI

```bash
agent-rig increment44 --out artifacts/increment44
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment44-threejs.sh
```

Writes `artifacts/increment44/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores the spring; bodies still use our post-step dump poses (settled cork, fallen bob, yawed gate, ridden rider, slid platform, ball off its seat, seated lid, closed drawer).



## Increment 45

Same courtyard as increment 44 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]`, brass `bob` on a breakable `gate`–`bob` rope, tan `cork` on a gate spring). `Joint::Hinge` gains optional `motor_target_position` (radians, serde default none). The increment-45 ground–gate hinge keeps limits `[0, 1.15]` and `motor_max_force` 5.0 but replaces velocity 1.4 with `motor_target_position` ~0.55 (no `motor_target_velocity`, or 0). After 120 steps the gate angle is within 0.15 of 0.55 — half-open, not parked at the 1.15 limit. Dump records `motor_target_position` on that hinge. `increment44_scene()` stays velocity-driven (`motor_target_velocity` 1.4, no `motor_target_position`) and its dump gate angle stays ~1.15. Increment 18–44 scene JSON is unchanged (no `motor_target_position`). Three.js ignores motors and uses the dump gate pose (half-open vs 44 fully open). No new lights. Camera stays increment-44 (one frame). No new bodies.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment45.sh
```

Writes:

- `artifacts/increment45/scene.json` — increment-44 courtyard; ground–gate hinge uses a position motor to 0.55
- `artifacts/increment45/physics.json` — post-step dump (poses, contacts, collider kinds, joints including the gate hinge `motor_target_position` + angle ~0.55, `broken_joints`, overlaps, `ray_hits`, `sweep_hits`, authored `impulses`, platform `kinematic: true`, fallen bob, settled cork)
- `artifacts/increment45/frame.png` — our renderer, 800x450, courtyard plus the half-open gate
- `artifacts/increment45/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); gate from dump pose

### CLI

```bash
agent-rig increment45 --out artifacts/increment45
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment45-threejs.sh
```

Writes `artifacts/increment45/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores motors; bodies still use our post-step dump poses (half-open gate, settled cork, fallen bob, ridden rider, slid platform, ball off its seat, seated lid, closed drawer).


## Increment 46

Same courtyard as increment 45 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]` driven to `motor_target_position` ~0.55, brass `bob` on a breakable `gate`–`bob` rope, tan `cork` on a gate spring). Increment 46 changes ONLY the single camera: position `[1.85, 1.35, 3.15]`, look_at `[0.35, 0.42, 1.55]`, `fov_y_deg` 40 — aimed at the gate / cork / fallen-bob cluster. Still one `camera`, one `frame.png`. No `cameras[]` array. No extra files. `increment45_scene()` keeps the wide courtyard camera `[3.6, 2.35, 5.2]` look_at `[0.1, 0.38, 0]`. Increment 18–45 scene JSON is unchanged (old camera 3.6, not 1.85). Physics is the same as 45: after 120 steps gate angle is within 0.15 of 0.55, cork spring present, bob on the floor, `broken_joints` gate–bob. No new bodies, lights, joints, or impulses. Gate + cork fill the frame (large pixel delta vs 45 is expected). Ours still wins vs Three.js on materials.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment46.sh
```

Writes:

- `artifacts/increment46/scene.json` — increment-45 courtyard; single camera re-aimed at the gate cluster
- `artifacts/increment46/physics.json` — post-step dump (same physics as increment 45: gate angle ~0.55, cork spring, fallen bob, `broken_joints`)
- `artifacts/increment46/frame.png` — our renderer, 800x450, gate + cork fill the frame
- `artifacts/increment46/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); gate from dump pose

### CLI

```bash
agent-rig increment46 --out artifacts/increment46
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment46-threejs.sh
```

Writes `artifacts/increment46/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores motors; bodies still use our post-step dump poses (half-open gate, settled cork, fallen bob, ridden rider, slid platform, ball off its seat, seated lid, closed drawer). Camera is the increment-46 close-up.


## Increment 47

Same courtyard as increment 46 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]` driven to `motor_target_position` ~0.55, brass `bob` on a breakable `gate`–`bob` rope, tan `cork` on a gate spring, camera `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]`). Increment 47 changes nothing visual: `increment47_scene()` clones `increment46_scene()` and only sets `record_contact_events`. No new lights, bodies, joints, impulses, or cameras. After 120 steps the physics dump records `contact_events: [{ "kind": "started"|"stopped", "body_a", "body_b" }]` collected across every step (not just the final `contacts` snapshot), including at least one `started` involving `bob` and `ground` and one `started` involving `rider` and `platform`. Final `contacts` stay. `increment46_scene()` and the increment-46 dump stay event-free (`contact_events` omitted when empty). Increment 18–46 scene JSON is unchanged (no `record_contact_events` / no `contact_events`). Three.js ignores `contact_events` and uses dump poses. Visual delta vs 46 can be tiny (dump increment). Gate angle ~0.55, cork spring, broken rope, fallen bob, courtyard stay.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment47.sh
```

Writes:

- `artifacts/increment47/scene.json` — increment-46 courtyard; `record_contact_events: true`
- `artifacts/increment47/physics.json` — post-step dump plus `contact_events` across the 120 steps (gate angle ~0.55, cork spring, fallen bob, `broken_joints`, final `contacts`)
- `artifacts/increment47/frame.png` — our renderer, 800x450, same gate-cluster camera as increment 46
- `artifacts/increment47/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); ignores `contact_events`

### CLI

```bash
agent-rig increment47 --out artifacts/increment47
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment47-threejs.sh
```

Writes `artifacts/increment47/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores contact events and motors; bodies still use our post-step dump poses (half-open gate, settled cork, fallen bob, ridden rider, slid platform, ball off its seat, seated lid, closed drawer). Camera is the increment-46 close-up.




## Increment 48

Same courtyard as increment 47 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]` driven to `motor_target_position` ~0.55, brass `bob` on a breakable `gate`–`bob` rope, tan `cork` on a gate spring, camera `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]`, `record_contact_events`). Increment 48 clones `increment47_scene()` and adds ONLY a coral `walker` box (`size` `[0.18, 0.36, 0.18]`, pose `[1.15, 0.20, 1.45]`, mass 0, albedo `[0.85, 0.22, 0.48]`) with `controller: { "desired_velocity": [-0.55, 0, 0] }`. No increment-39 `kinematic` flag on the walker (platform stays velocity-based). Each step Rapier `KinematicCharacterController::move_shape` slides + snaps the walker to the floor. After 120 steps walker.x has moved at least 0.4 toward −X, y stays on the floor (~0.14–0.28), and the dump records `controllers: [{ id: "walker", grounded: true, desired_velocity, effective_translation }]`. `increment47_scene()` stays walker-free; increment 18–47 scene JSON is unchanged (no `controller` key); increment-47 dump omits `controllers` when empty. Camera unchanged. No new shape types, no second camera, no collision groups. Three.js uses dump poses (it does not run the controller). Walker is visible in the tight gate-cluster shot.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment48.sh
```

Writes:

- `artifacts/increment48/scene.json` — increment-47 courtyard plus the coral walker + controller
- `artifacts/increment48/physics.json` — post-step dump plus `controllers` (walker grounded, last-step effective translation) and `contact_events` (gate angle ~0.55, cork spring, fallen bob, `broken_joints`)
- `artifacts/increment48/frame.png` — our renderer, 800x450, coral walker in the gate-cluster camera
- `artifacts/increment48/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); walker from dump pose

### CLI

```bash
agent-rig increment48 --out artifacts/increment48
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment48-threejs.sh
```

Writes `artifacts/increment48/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores the character controller and contact events; bodies still use our post-step dump poses (walked walker, half-open gate, settled cork, fallen bob, ridden rider, slid platform, ball off its seat, seated lid, closed drawer). Camera is the increment-46 close-up.



## Increment 49

Same courtyard as increment 48 (bowl + rock + gold ball with clearcoat / anisotropy / iridescence, copper pillar with AO, glass pane with transmission + IOR + volume + dispersion, crate, sheen bench, hanging lantern + hinge motor 4/8, drawer + slider motor -2/6, charm + ball socket, drawer-open trigger, emissive lantern 16, directional + area light, `drawer_probe` raycast, `drawer_sweep` shapecast, crate lid + fixed joint, ball impulse `[1.8, 0.4, 0.5]`, kinematic `platform` sliding +X at `[0.45, 0, 0]`, dynamic clay `rider`, teal `gate` on a limited hinge `[0, 1.15]` driven to `motor_target_position` ~0.55, brass `bob` on a breakable `gate`–`bob` rope, tan `cork` on a gate spring, coral `walker` with `controller.desired_velocity` `[-0.55, 0, 0]`, camera `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]`, `record_contact_events`). Increment 49 clones `increment48_scene()` and adds ONLY a yellow `bar` box (`size` `[0.08, 0.40, 0.28]`, pose `[0.55, 0.22, 1.45]`, mass 0, albedo `[0.92, 0.78, 0.18]`) plus `collision_groups` on `walker` (`membership` 2 / `filter` 1 — stands on ground, ignores the bar) and `bar` (`membership` 4 / `filter` `0xFFFF`). Ground stays default all-bits (includes GROUND=1). After 120 steps walker.x has moved at least 0.4 toward −X (walks through the bar), y stays on the floor (~0.14–0.28), grounded is true, the bar stays at its authored pose, dump body states for walker and bar include `collision_groups`, and `contact_events` has no walker–bar started pair. `increment48_scene()` stays bar-free and group-free; increment 18–48 scene JSON is unchanged (no `collision_groups` key); increment-48 dump omits `collision_groups` when default. Camera, walker start, and walker controller unchanged. No spawn/despawn, no pickups, no follow-cam, no new shape types, no second camera. Three.js uses dump poses.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment49.sh
```

Writes:

- `artifacts/increment49/scene.json` — increment-48 courtyard plus the yellow bar + collision groups
- `artifacts/increment49/physics.json` — post-step dump plus walker/bar `collision_groups`, `controllers` (walker grounded), and `contact_events` (no walker–bar started; gate angle ~0.55, cork spring, fallen bob, `broken_joints`)
- `artifacts/increment49/frame.png` — our renderer, 800x450, yellow bar + coral walker past/through it in the gate-cluster camera
- `artifacts/increment49/threejs-frame.png` — stock Three.js, same pose (`MeshPhysicalMaterial`, no env map, no tonemap); walker and bar from dump poses

### CLI

```bash
agent-rig increment49 --out artifacts/increment49
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment49-threejs.sh
```

Writes `artifacts/increment49/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js ignores collision groups, the character controller, and contact events; bodies still use our post-step dump poses (walked-through-bar walker, unmoved yellow bar, half-open gate, settled cork, fallen bob, ridden rider, slid platform, ball off its seat, seated lid, closed drawer). Camera is the increment-46 close-up.



## Increment 50

Same courtyard as increment 49 (yellow `bar`, coral `walker` with collision groups, gate-cluster camera). Increment 50 clones `increment49_scene()` and adds ONLY `spawns` + `despawns`: gold `token` sphere (radius 0.10, pose `[0.70, 0.12, 1.45]`, mass 0, albedo `[0.95, 0.78, 0.22]`, groups membership 4 / filter `0xFFFF`) at step 30, and despawn `bar` at step 80. After 120 steps the dump includes `token` near its authored pose, does not include `bar`, `spawned` has `{ id: "token", at_step: 30 }`, and `despawned` has `{ id: "bar", at_step: 80 }`. Walker still walks Δx ≥ 0.4, y on the floor, grounded. `increment49_scene()` stays timed-event-free (bar still there, no token); increment 18–49 scene JSON is unchanged (no `spawns` / `despawns` keys); increment-49 dump omits `spawned` / `despawned` when empty. Camera, walker, collision groups, courtyard leftovers unchanged. No pickup-on-overlap, no follow-cam, no play-until, no second scene. Three.js uses dump poses.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment50.sh
```

Writes:

- `artifacts/increment50/scene.json` — increment-49 courtyard plus scheduled spawn/despawn
- `artifacts/increment50/physics.json` — post-step dump plus `spawned` (token@30) and `despawned` (bar@80); token present, bar gone
- `artifacts/increment50/frame.png` — our renderer, 800x450, gold token in / yellow bar gone in the gate-cluster camera
- `artifacts/increment50/threejs-frame.png` — stock Three.js, same pose; token from dump poses

### CLI

```bash
agent-rig increment50 --out artifacts/increment50
```

### Three.js baseline (same post-step pose)

```bash
cd /workspace/agent-rig && ./scripts/increment50-threejs.sh
```

Writes `artifacts/increment50/threejs-frame.png` (stock MeshPhysicalMaterial, ambient 0.25 + directional + RectAreaLight, lantern emissive, no env map, no tonemap, 800x450). Three.js uses dump poses: gold token present, yellow bar gone, walked walker, half-open gate, settled cork, fallen bob, ridden rider, slid platform. Camera is the increment-46 close-up.


## Increment 51

Same courtyard as increment 50 (gold token spawn@30, yellow bar despawn@80, coral walker, gate-cluster camera). Increment 51 clones `increment50_scene()` and adds ONLY trigger `token_zone` (box `[0.40, 0.40, 0.40]` at `[0.70, 0.12, 1.45]`, sensor) and `pickups: [{ body: token, trigger: token_zone, by: walker }]`. After the token spawns, the walker overlapping `token_zone` despawns the token as a pickup. After 120 steps the dump has no token, `picked_up` includes `{ id: token, by: walker, at_step in 30–80 }`, `spawned` still has token@30, and `despawned` still has bar@80 only. `increment50_scene()` stays pickup-free (token still there, no `token_zone`); increment 18–50 scene JSON is unchanged (no `pickups` key); increment-50 dump omits `picked_up` when empty. Camera, walker, spawn/despawn, courtyard leftovers unchanged. No follow-cam, no play-until, no second scene. Three.js uses dump poses (token gone).

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment51.sh
```

Writes:

- `artifacts/increment51/scene.json` — increment-50 courtyard plus token_zone + pickups
- `artifacts/increment51/physics.json` — post-step dump plus `picked_up` (token by walker); token gone, bar gone
- `artifacts/increment51/frame.png` — our renderer, 800x450, gold token gone vs increment 50
- `artifacts/increment51/threejs-frame.png` — stock Three.js, same pose; no token

### CLI

```bash
cargo run --release -- increment51 --out artifacts/increment51
```


## Increment 52

Same courtyard as increment 51 (gold token spawn@30 / pickup, yellow bar despawn@80, coral walker). Increment 52 clones `increment51_scene()` and adds ONLY `camera.follow` `{ body: walker, offset: [1.20, 0.90, 1.50] }`. Authored rest camera stays `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]` fov 40. After 120 steps the dump records `camera` from the walker pose + offset. `increment51_scene()` stays follow-free (no `follow` key, dump has no `camera` key); increment 18–51 scene JSON is unchanged. Pickup leftovers stay (no token, picked_up token by walker, spawned token@30, despawned bar@80). Three.js uses dump.camera when present. No play-until.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment52.sh
```

Writes:

- `artifacts/increment52/scene.json` — increment-51 courtyard plus camera.follow
- `artifacts/increment52/physics.json` — post-step dump plus resolved `camera`; token gone, bar gone
- `artifacts/increment52/frame.png` — our renderer, 800x450, follow-cam on the walker
- `artifacts/increment52/threejs-frame.png` — stock Three.js, same pose and dump.camera

### CLI

```bash
cargo run --release -- increment52 --out artifacts/increment52
```


## Increment 53

Same courtyard as increment 52 (follow-cam on the walker, gold token spawn@30 / pickup, yellow bar despawn@80). Increment 53 clones `increment52_scene()` and adds ONLY `play_until` `{ kind: picked_up, body: token }`. `--steps` is a max cap; the sim stops when the token is picked. After play-until the dump has `steps` 30–31, `stopped: { kind: picked_up, body: token }`, no token, yellow bar still present (despawn is at 80), `picked_up` token by walker, `spawned` token@30, empty `despawned`, and follow-cam from the walker at pickup (not the increment-52 120-step camera). `increment52_scene()` stays fixed-step (no `play_until`, dump omits `stopped`); increment 18–52 scene JSON is unchanged (no `play_until` key). Follow-cam, pickup, walker, courtyard leftovers unchanged. Three.js uses dump poses and dump.camera.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment53.sh
```

Writes:

- `artifacts/increment53/scene.json` — increment-52 courtyard plus play_until
- `artifacts/increment53/physics.json` — dump stopped at token pickup (`steps` 30–31, `stopped` picked_up/token); token gone, bar present
- `artifacts/increment53/frame.png` — our renderer, 800x450, follow-cam at pickup, yellow bar still in frame
- `artifacts/increment53/threejs-frame.png` — stock Three.js, same pose and dump.camera

### CLI

```bash
cargo run --release -- increment53 --out artifacts/increment53
```


## Increment 54

A second scene: a short stone lane, not the courtyard. `increment54_scene()` is authored from scratch (not `increment53_scene().clone()`). Coral walker walks +x to a gold token present from t=0; `play_until` `{ kind: picked_up, body: token }` stops the sim on pickup. Bodies are only ground (stone box), walker, token, and an optional teal block for scale — no courtyard leftovers. Follow-cam offset `[-1.00, 0.80, 1.60]`. `increment53_scene()` stays the courtyard. Increment 18–53 scene JSON is unchanged (no lane rest camera, no lane ground size). Three.js uses dump.camera when present.

### One command

```bash
cd /workspace/agent-rig && ./scripts/increment54.sh
```

Writes:

- `artifacts/increment54/scene.json` — short stone lane with play_until and follow-cam
- `artifacts/increment54/physics.json` — dump stopped at token pickup (`steps` 30–110, `stopped` picked_up/token); token gone, walker + ground present
- `artifacts/increment54/frame.png` — our renderer, 800x450, follow-cam at pickup
- `artifacts/increment54/threejs-frame.png` — stock Three.js, same pose and dump.camera

### CLI

```bash
cargo run --release -- increment54 --out artifacts/increment54
```

## Scene format

JSON object with `camera`, `lights`, `bodies`, optional `joints`, optional `triggers`, optional `raycasts`, optional `shapecasts`, optional `impulses`, optional `record_contact_events`, optional `spawns`, and optional `despawns`.

- `camera`: `position`, `look_at`, `fov_y_deg`
- `lights`: `type: "directional"` with `direction`, `color`, `intensity`; or `type: "point"` with `position`, `color`, `intensity`; or `type: "area"` with `position`, `size` `[width, height]`, `color`, `intensity`, optional `normal` (default `[0,-1,0]`). Area softness comes from the authored `size`.
- `bodies`: `id`, `shape` (`box` + full `size` xyz, `sphere` + `radius`, or `mesh` + `path` + `collider`), `position`, optional `rotation_wxyz` (default `[1,0,0,0]`), `mass` (0 = static), optional `linear_velocity`, optional `kinematic` (bool, serde default false, omitted when false; when true spawn as Rapier `kinematic_velocity_based` and re-apply authored `linear_velocity` every step), optional `controller` (`{ "desired_velocity": [x,y,z] }` world m/s, serde default none, omitted when none; when present spawn as Rapier `kinematic_position_based` and drive each step with `KinematicCharacterController::move_shape`, excluding self — do not also set `kinematic: true`), optional `collision_groups` (`{ "membership": u32, "filter": u32 }` Rapier InteractionGroups, serde default `0xFFFF` / `0xFFFF`, omitted when default), `material` (`albedo`, `roughness`, `metallic`, optional `albedo_map` PNG path, optional `clearcoat` + `clearcoat_roughness`, optional `sheen` + `sheen_roughness` + `sheen_color`, optional `anisotropy` + `anisotropy_rotation`, optional `iridescence` + `iridescence_ior` + `iridescence_thickness`, optional `dispersion`, optional `emissive` + `emissive_intensity`)
- `triggers` (optional, default empty): `{ "id", "shape": { "type": "box", "size": [x,y,z] }, "position": [x,y,z] }`. Mapped to a Rapier sensor cuboid (`sensor = true`) on a fixed body. Sensors report overlaps and do not generate contact forces or push bodies. After stepping, the dump records `overlaps: [{ "trigger", "body" }]`. Rendered as a translucent cyan box (BLEND, alpha ~0.18).
- `raycasts` (optional, default empty): `{ "id", "origin", "direction", "max_toi" }`. After stepping, the dump records `ray_hits: [{ "ray", "body", "point", "normal", "toi" }]` (hits only; misses omitted). Sensors are skipped. Rendered as a thin magenta segment plus a hit marker.
- `shapecasts` (optional, default empty): `{ "id", "origin", "direction", "shape": { "type": "box", "size": [x,y,z] }, "max_toi" }`. Swept as a Rapier cuboid (half-extents = size/2) with `cast_shape`. After stepping, the dump records `sweep_hits: [{ "sweep", "body", "point", "normal", "toi" }]` (hits only; misses omitted). Sensors are skipped so a trigger cannot steal the hit. Rendered as a translucent box at the hit pose (`origin + dir * toi`) plus a small hit marker; on a miss the box is drawn at `origin + dir * max_toi`.
- `impulses` (optional, default empty): `{ "body", "linear": [x,y,z] }`. Applied once at spawn via Rapier `apply_impulse` in world space at the center of mass (no `point` this increment). Not applied every step. After stepping, the dump records the authored impulses (not post-step velocity). Three.js ignores impulses; bodies use dump poses.
- `record_contact_events` (optional bool, serde default false, omitted when false). When true, Rapier collision events are collected across every physics step and the dump records `contact_events: [{ "kind": "started"|"stopped", "body_a", "body_b" }]`. The final `contacts` snapshot is unchanged. Increment 46 and earlier stay event-free (field absent). Three.js ignores `contact_events`; bodies use dump poses.
- `controller` (optional on a body, serde default none). `{ "desired_velocity": [x, y, z] }` is a world-space wish in m/s. The engine adds a downward component (gravity and/or `snap_to_ground`) so the walker stays on the floor. After stepping, the dump records `controllers: [{ "id", "grounded", "desired_velocity", "effective_translation" }]` (last-step translation is fine). Omitted when empty so increment-47 dumps have no `controllers` key. Three.js ignores the controller and uses dump poses.
- `collision_groups` (optional on a body, serde default `{ "membership": 65535, "filter": 65535 }`, omitted when default). Rapier `InteractionGroups`: collision iff `(a.membership & b.filter) != 0 && (b.membership & a.filter) != 0`. Mapped to `collider.set_collision_groups`. Character-controller `move_shape` uses a `QueryFilter` with those groups so a walker whose filter is GROUND=1 slides through a PROP=4 bar. After stepping, the dump records `collision_groups` on body states when non-default (increment-48 dumps have no key). Three.js ignores groups and uses dump poses.
- `spawns` (optional, default empty, omitted when empty): `{ "at_step": u32, "body": <Body> }`. At the start of that 0-based step (before `pipeline.step` / `move_shape`), insert the body into the Rapier world on the same path as initial spawn. After stepping, the dump records `spawned: [{ "id", "at_step" }]`. Increment 18-49 stay without this key.
- `despawns` (optional, default empty, omitted when empty): `{ "at_step": u32, "body": "id" }`. At the start of that 0-based step, remove the rigid body + colliders, drop joints attached to it, and drop `collider_to_id` / handles. After stepping, the dump records `despawned: [{ "id", "at_step" }]`. Increment 18-49 stay without this key.
- `joints` (optional, default empty): `{ "type": "hinge", "body_a", "body_b", "anchor": [x,y,z], "axis": [x,y,z] }` with optional `limits` `[min, max]` radians (serde default none = unlimited; skipped when serializing so old hinges stay compact) and optional `motor_target_velocity` + `motor_max_force` (serde default 0; both 0 = hang damper) and optional `motor_target_position` (radians, serde default none; when Some and motor_max_force > 0, Rapier position motor instead of velocity), or `{ "type": "slider", "body_a", "body_b", "axis": [x,y,z], "limits": [min, max] }` with optional `anchor` (closed-pose world attachment) and optional `motor_target_velocity` + `motor_max_force` (serde default 0; both 0 = no motor, increment 29–33 open from authored velocity), or `{ "type": "ball", "body_a", "body_b", "anchor": [x,y,z] }`, or `{ "type": "fixed", "body_a", "body_b", "anchor": [x,y,z] }`, or `{ "type": "distance", "body_a", "body_b", "anchor": [x,y,z], "rest_length" }` with optional `break_force` (serde default 0 = never break; skipped when serializing so increment-42 ropes stay compact). or `{ "type": "spring", "body_a", "body_b", "anchor": [x,y,z], "rest_length", "stiffness", "damping" }`. `anchor` is world-space (converted to local on each body at spawn). `axis` is a world-space direction; hinge axes should be horizontal so gravity hangs the child. Mapped to a Rapier revolute, prismatic, spherical, fixed (weld), rope (max length = `rest_length`), or spring (rest length / stiffness / damping) impulse joint. Authored hinge `limits` map to Rapier `RevoluteJointBuilder::limits([min, max])`. A nonzero hinge motor replaces the hang damper with `motor_velocity(target, max_force)`. When a hinge authors `motor_target_position` and `motor_max_force` > 0, Rapier `motor_position(target, stiffness, damping)` drives the angle instead of velocity. A nonzero slider motor drives along the axis with `motor_velocity(target, max_force)` (ForceBased). A ball socket is free in 2 axes (not locked to a hinge swing plane). A fixed joint locks all relative degrees of freedom so the child stays seated on the parent. A distance / rope joint limits the maximum separation between the two local anchors to the authored `rest_length` so gravity can hang the child. Each step, if a distance joint's reaction magnitude exceeds `break_force` (> 0), the impulse joint is removed; the dump omits it from `joints` and records `broken_joints: [{ "kind", "body_a", "body_b" }]`. A spring joint applies a ForceBased spring-damper between the two local anchors (Rapier `SpringJoint`) so gravity can hang the child near the authored `rest_length`.
- mesh `path` (`.obj`, `.gltf`, or `.glb`) and `albedo_map` are relative to the repo root (or the scene file). `collider` is `convex_hull` or `trimesh`. If `albedo_map` is set, the sampled texel replaces albedo on that body. glTF load is POSITION + optional TEXCOORD_0 + indices, TRIANGLES only. If the primitive has `pbrMetallicRoughness` (`baseColorFactor` and/or `baseColorTexture`, plus optional metallic/roughness factors and optional `metallicRoughnessTexture`), that drives the look; scene JSON `material` is the fallback when the glTF has no material. When `metallicRoughnessTexture` is present, roughness is sampled from G and metallic from B (times the factors); scene-JSON metallic/roughness do not override the texel. When `normalTexture` is present, RGB is unpacked to a tangent-space normal (2c−1) and TBN·n_ts replaces the geometric N for lighting (TANGENT accessor, or TBN derived from triangle positions + TEXCOORD_0). Scene JSON has no normal map. When `emissiveFactor` and/or `emissiveTexture` are present, sampled emissive is `factor * textureRGB` (no texture → factor only) and is added to outgoing radiance after lighting. Scene JSON has no emissive map. Optional scene-JSON `emissive` + `emissive_intensity` (serde default 0 = off) add self-glow (`emissive × intensity`) and treat that body's post-step COM as a point / mesh light on other surfaces; intensity 0 keeps increment-16 texture emissive with no mesh light. This is not a new `lights[]` entry. When `alphaMode` is `BLEND`, the surface is shaded then the ray continues and the results are blended (`src * alpha + behind * (1-alpha)`); `MASK` discards below `alphaCutoff`. Alpha is `baseColorFactor[3]` times baseColorTexture A if present. When `KHR_materials_transmission.transmissionFactor` is > 0, the continuation uses Snell refraction with the authored IOR (`materials.ior` or `KHR_materials_ior`, glass ~1.5): `eta = 1/ior` entering, `ior` leaving. No hidden bend constant. When `KHR_materials_volume` is present, the enter→exit path through the volume applies Beer-Lambert: `T = attenuationColor.pow(distance / attenuationDistance)` per channel, multiplied onto the radiance behind the pane. `attenuationColor` and `attenuationDistance` are authored (not hidden constants); this is volume absorption, not a change to `baseColorFactor`. When `occlusionTexture` is present, the R channel is sampled as AO (0 = occluded, 1 = open) and multiplies IBL / ambient; directional and area direct lighting are not multiplied by AO. Scene JSON has no AO map. Optional `anisotropy` (0–1) and `anisotropy_rotation` (radians) on a body material stretch the specular into a brushed-metal GGX lobe; strength and direction are the authored values (default 0 = isotropic). Optional `iridescence` (0–1), `iridescence_ior` (default 1.3), and `iridescence_thickness` (nm, default 400) add a thin-film rainbow on the specular F0 / Fresnel; factor, IOR, and thickness are the authored values (default 0 = off). Optional `dispersion` (KHR_materials_dispersion, default 0) on a transmitting material splits refracted rays into R/G/B with Cauchy IOR `n(λ) = ior + dispersion * (1/λ² − 1/0.55²)`; strength is the authored value (0 = increment-26 single-ray refraction).

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

Increment 33: existing lantern authors `emissive ≈ [1.0, 0.55, 0.12]` and `emissive_intensity ≈ 16`; increment 32 lantern stays dark (intensity 0); increment 18–32 scene JSON unchanged (no lantern emissive_intensity 16); courtyard + drawer_open trigger stay; dump records the overlap plus ball + hinge motor + slider; pillar `convex_hull` + ground–pillar contacts stay; no extra `lights[]` entries vs increment 32; `increment33` writes scene + physics dump + our PNG.

Increment 34: existing crate–drawer slider authors `motor_target_velocity ≈ -2` and `motor_max_force ≈ 6`; increment 33 slider stays 0 / no motor fields; increment 18–33 scene JSON unchanged; after stepping drawer COM z < 1.15 (closed vs increment 33's ~1.375); dump records slider motor fields plus hinge motor 4/8 + ball; trigger stays authored (overlap may be empty); pillar `convex_hull` + ground–pillar contacts stay; courtyard (emissive lantern 16, trigger, charm + ball, crate, sheen bench, gold clearcoat+anisotropy+iridescence, volume+dispersion pane, area light, AO) stays; `increment34` writes scene + physics dump + our PNG.

Increment 35: scene authors one ray `drawer_probe`; increment 34 stays raycast-free; increment 18-34 scene JSON unchanged; dump records a hit on the drawer; courtyard + motors stay; `increment35` writes scene + physics dump + our PNG.

Increment 36: scene authors one shapecast `drawer_sweep`; increment 35 stays shapecast-free and keeps `drawer_probe`; increment 18-35 scene JSON unchanged; dump records a sweep hit on the drawer and still has `ray_hits` drawer_probe → drawer; courtyard + motors stay; `increment36` writes scene + physics dump + our PNG.

Increment 37: scene authors one crate lid + a Rapier fixed / weld joint (`type: "fixed"`, crate–lid, world-space anchor); increment 36 stays lid-free and fixed-joint-free; increment 18-36 scene JSON unchanged (no `"fixed"`, no lid body); after stepping the lid y stays near authored `~0.28` (not on the ground); dump records `kind: "fixed"` crate/lid AND still has hinge/slider/ball + `ray_hits` drawer_probe + `sweep_hits` drawer_sweep; courtyard + motors stay; `increment37` writes scene + physics dump + our PNG.

Increment 38: scene authors one impulse on the gold ball (`body: "ball"`, `linear: [1.8, 0.4, 0.5]`); increment 37 stays impulse-free; increment 18-37 scene JSON unchanged (no `"impulses"`); applied once at spawn (world-space, at COM); after 120 steps `|ball.x + 1.1| > 0.25` (rolls off the increment-37 seat); dump records the authored impulse (not post-step velocity) AND still has lid + fixed + hinge/slider/ball + `ray_hits` drawer_probe + `sweep_hits` drawer_sweep; courtyard + motors stay; `increment38` writes scene + physics dump + our PNG.

Increment 38: scene authors one impulse on the gold ball (`linear: [1.8, 0.4, 0.5]`, applied once at spawn at COM); increment 37 stays impulse-free; increment 18-37 scene JSON unchanged (no `impulses` key); after stepping |ball.x + 1.1| > 0.25 (rolls off the seat); dump records the authored impulse; lid + fixed + drawer_probe + drawer_sweep + motors stay; `increment38` writes scene + physics dump + our PNG.

Increment 39: scene authors one kinematic `platform` (box `[0.55, 0.06, 0.35]`, pose `[-0.55, 0.04, -0.55]`, `kinematic: true`, `linear_velocity: [0.45, 0, 0]`); increment 38 stays platform-free and kinematic-free; increment 18-38 scene JSON unchanged (no `"kinematic"`, no `"platform"`); spawned as Rapier `KinematicVelocityBased` and driven each step from authored velocity; after 120 steps `|platform.x - authored_x| > 0.4`; dump records the platform pose AND `kinematic: true`; lid + fixed + ball impulse + drawer_probe + drawer_sweep + motors stay; `increment39` writes scene + physics dump + our PNG.

Increment 40: scene authors one dynamic `rider` (box `[0.16, 0.16, 0.16]`, pose `[-0.55, 0.15, -0.55]`, mass `0.35`, clay albedo, not kinematic) on the kinematic platform; increment 39 stays rider-free; increment 18-39 scene JSON unchanged (no `"rider"`); after 120 steps the rider has ridden +X and stays on the slab; dump records rider + platform `kinematic: true`; courtyard + motors stay; `increment40` writes scene + physics dump + our PNG.

Increment 41: `Joint::Hinge` gains optional `limits: [min, max]` (radians, serde default none = unlimited); scene authors one teal `gate` + `ground`–`gate` hinge with `limits` `[0, 1.15]` and motor `1.4/5.0`; increment 40 stays gate-free and the pillar–lantern hinge stays limit-free; increment 18-40 scene JSON unchanged (no `"gate"`, no hinge `"limits"`); after 120 steps the gate has yawed and the hinge angle is within 0.2 of 1.15 and ≤ 1.30; dump records `limits` (and angle) on that hinge; courtyard + rider + platform + motors stay; `increment41` writes scene + physics dump + our PNG.

Increment 42: `Joint::Distance` (`type: "distance"`, world `anchor` + `rest_length`) maps to a Rapier `RopeJoint` (max length = rest_length); scene authors one brass `bob` hung from the gate; increment 41 stays bob-free and distance-joint-free; increment 18-41 scene JSON unchanged (no `"distance"`, no `"bob"`); after 120 steps bob.y < 0.88 − 0.12 and |bob COM − current gate-top| ≤ 0.38 + 0.08; dump records `kind: "distance"` plus `rest_length`; courtyard + gate limits + rider + platform + motors stay; `increment42` writes scene + physics dump + our PNG.

Increment 43: `Joint::Distance` gains optional `break_force` (serde default 0 = never break); scene clones increment 42 and only sets `break_force` ~1.5 on the gate–bob rope plus one extra bob impulse `[0.0, -4.0, 1.6]`; increment 42 stays unbreakable (no `break_force` / 0, no bob impulse) and the bob still hangs; increment 18-42 scene JSON unchanged (no `"break_force"`); after 120 steps bob.y < 0.22 (on the bowl), dump `joints` has no live gate–bob distance, and `broken_joints` includes `{kind:distance, body_a:gate, body_b:bob}`; courtyard + gate limits + rider + platform + motors stay; `increment43` writes scene + physics dump + our PNG.

Increment 44: `Joint::Spring` (`type: "spring"`, world `anchor` + `rest_length` + `stiffness` + `damping`) maps to a Rapier `SpringJoint`; scene authors one tan `cork` hung from the gate; increment 43 stays cork-free and spring-free; increment 18-43 scene JSON unchanged (no `"spring"`, no `"cork"`); after 120 steps cork.y < 1.15 - 0.15 and |cork COM - current gate-top - 0.42| <= 0.12; dump records `kind: "spring"` plus `rest_length`/`stiffness`/`damping`; broken rope + fallen bob + courtyard stay; `increment44` writes scene + physics dump + our PNG.

Increment 45: `Joint::Hinge` gains optional `motor_target_position` (radians, serde default none); increment-45 gate hinge keeps limits [0, 1.15] and motor_max_force 5.0 but replaces velocity 1.4 with target ~0.55; increment44_scene stays velocity-driven (1.4, no target) and its dump angle stays ~1.15; increment 18-44 scene JSON unchanged (no `motor_target_position`); after 120 steps gate angle is within 0.15 of 0.55 (not parked at 1.15); dump records `motor_target_position` on that hinge; courtyard (cork spring, broken rope + fallen bob, rider, platform, lid + fixed, impulses, drawer_probe, drawer_sweep) stays; `increment45` writes scene + physics dump + our PNG.

Increment 46: increment46_scene clones increment45 and changes ONLY the camera (position `[1.85, 1.35, 3.15]`, look_at `[0.35, 0.42, 1.55]`, fov_y_deg 40) aimed at the gate / cork / fallen-bob cluster; increment45_scene keeps `[3.6, 2.35, 5.2]` look_at `[0.1, 0.38, 0]`; increment 18-45 scene JSON unchanged (old camera 3.6, not 1.85); still one `camera`, one `frame.png`, no `cameras[]`; physics same as 45 (gate angle within 0.15 of 0.55, cork spring, bob y < 0.22, broken_joints gate–bob); no new bodies / lights / joints / impulses; `increment46` writes scene + physics dump + our PNG.

Increment 47: increment47_scene clones increment46 and changes nothing visual (same camera `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]`, same bodies / joints / impulses); only sets `record_contact_events`; increment46_scene and increment 18-46 scene JSON stay without `record_contact_events` / `contact_events`; after 120 steps the dump records `contact_events` across the step loop including started bob+ground and started rider+platform, and still has the final `contacts` snapshot; courtyard stays (gate angle ~0.55, cork spring, bob y < 0.22, broken_joints gate–bob); Three.js ignores contact_events; `increment47` writes scene + physics dump + our PNG.

Increment 48: increment48_scene clones increment47 and adds ONLY `walker` + `controller.desired_velocity` ≈ `[-0.55, 0, 0]` (no increment-39 `kinematic` flag); increment47_scene stays walker-free and controller-free; increment 18-47 scene JSON unchanged (no `"controller"`); after 120 steps walker.x ≤ authored_x − 0.4, walker.y in ~[0.14, 0.28], dump.controllers includes `{ id: "walker", grounded: true, desired_velocity, effective_translation }`; increment47 dump has no `controllers` key; courtyard stays (contact_events, gate ~0.55, cork spring, bob y < 0.22, broken_joints, rider, platform kinematic, lid+fixed, impulses, drawer_probe, drawer_sweep); `increment48` writes scene + physics dump + our PNG.

Increment 49: increment49_scene clones increment48 and adds ONLY `bar` + `collision_groups` on walker (membership 2 / filter 1) and bar (membership 4 / filter 0xFFFF); increment48_scene stays bar-free and group-free; increment 18-48 scene JSON unchanged (no `"collision_groups"`); after 120 steps walker.x ≤ authored_x − 0.4 (walks through the bar), walker.y in ~[0.14, 0.28], grounded true, bar unmoved, dump walker/bar include `collision_groups`, contact_events has no walker–bar started pair; increment48 dump bodies have no `collision_groups` key; courtyard stays (walker controller, contact_events, gate ~0.55, cork spring, bob y < 0.22, broken_joints, rider, platform kinematic, lid+fixed, impulses, drawer_probe, drawer_sweep); `increment49` writes scene + physics dump + our PNG.

Increment 50: increment50_scene clones increment49 and adds ONLY `spawns` (gold token at step 30) + `despawns` (bar at step 80); increment49_scene stays timed-event-free (bar present, no token); increment 18-49 scene JSON unchanged (no `"spawns"` / `"despawns"`); after 120 steps dump.bodies has token near [0.70, 0.12, 1.45] and no bar, dump.spawned includes token@30, dump.despawned includes bar@80; increment49 dump has no spawned/despawned keys; walker still Δx ≥ 0.4, y on floor, grounded; courtyard stays; `increment50` writes scene + physics dump + our PNG.

Increment 51: increment51_scene clones increment50 and adds ONLY `token_zone` + `pickups` (token / token_zone / walker); increment50_scene stays pickup-free (token present, no token_zone); increment 18-50 scene JSON unchanged (no `"pickups"`); after 120 steps dump.bodies has no token, dump.picked_up includes token by walker at_step 30-80, dump.spawned still token@30, dump.despawned still bar@80 only; increment50 dump has token and no picked_up key; walker still Δx ≥ 0.4, y on floor, grounded; courtyard stays; `increment51` writes scene + physics dump + our PNG.

Increment 52: increment52_scene clones increment51 and adds ONLY `camera.follow` `{ body: walker, offset: [1.20, 0.90, 1.50] }`; authored rest camera stays `[1.85, 1.35, 3.15]` look_at `[0.35, 0.42, 1.55]` fov 40; increment51_scene stays follow-free (no `follow` key, dump has no `camera` key); increment 18-51 scene JSON unchanged; after 120 steps dump.camera.position ≈ `[1.249, 1.084, 2.949]` and look_at ≈ `[0.049, 0.334, 1.449]` from walker pose + offset; pickup leftovers stay (no token, picked_up token@30, spawned token@30, despawned bar@80); Three.js uses dump.camera when present; `increment52` writes scene + physics dump + our PNG.

Increment 53: increment53_scene clones increment52 and adds ONLY `play_until` `{ kind: picked_up, body: token }`; `--steps` is a max cap; increment52_scene stays fixed-step (no `play_until`, dump omits `stopped`); increment 18-52 scene JSON unchanged (no `play_until` key); dump.steps is 30..=31, dump.stopped is `{ kind: picked_up, body: token }`, bar still present, token gone, picked_up token by walker, spawned token@30, despawned empty; follow-cam is walker pose + offset at pickup (not increment-52 120-step camera numbers); `increment53` writes scene + physics dump + our PNG.
Increment 54: a second authored scene (short stone lane, not a courtyard clone); walker walks +x to a gold token present from t=0; play_until stops on pickup; increment53_scene stays the courtyard; increment 18-53 scene JSON unchanged; dump.steps 30..=110, dump.stopped picked_up/token, no token, walker+ground present, follow-cam; `increment54` writes scene + physics dump + our PNG.
