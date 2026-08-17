//! Agent-native scene + physics inspect + headless PNG (increments 1–65).

mod mesh;
mod physics;
mod render;
mod scene;

pub use physics::{simulate_trajectory, step_physics, step_physics_with_carry, step_catalog_scene, step_catalog_scene_with_carry, apply_win, BrokenJoint, ContactEvent, ControllerState, DespawnRecord, DumpCamera, DroppedRecord, HeldRecord, PhysicsBodyState, PhysicsContact, PhysicsDump, PhysicsOverlap, PhysicsJoint, PickupRecord, SpawnRecord, StoppedRecord, Trajectory, TrajectoryFrame, TransitionRecord, WonRecord, LostRecord, UsedRecord};
pub use render::{area_light_visibility, point_light_occluded, render_scene, render_scene_to_png, FRAME_HEIGHT, FRAME_WIDTH};
pub use mesh::{
    apply_tbn, load_gltf, load_mesh, load_obj, parse_obj, tbn_from_positions_uvs, GltfAlphaMode,
    GltfPbrMaterial, TriangleMesh,
};
pub use scene::{
    demo_scene, demo_scene_json, increment2_scene, increment2_scene_json, increment3_scene,
    increment3_scene_json, increment4_scene, increment4_scene_json, increment5_scene,
    increment5_scene_json, increment6_scene, increment6_scene_json, increment7_scene, increment7_scene_json, increment8_scene, increment8_scene_json, increment9_scene, increment9_scene_json, increment10_scene, increment10_scene_json, increment11_scene, increment11_scene_json, increment12_scene, increment12_scene_json, increment13_scene, increment13_scene_json, increment14_scene, increment14_scene_json, increment15_scene, increment15_scene_json, increment16_scene, increment16_scene_json, increment17_scene, increment17_scene_json, increment18_scene, increment18_scene_json, increment19_scene, increment19_scene_json, increment20_scene, increment20_scene_json, increment21_scene, increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene, increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene, increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene, increment27_scene_json, increment28_scene, increment28_scene_json, increment29_scene, increment29_scene_json, increment30_scene, increment30_scene_json, increment31_scene, increment31_scene_json, increment32_scene, increment32_scene_json, increment33_scene, increment33_scene_json, increment34_scene, increment34_scene_json, increment35_scene, increment35_scene_json, increment36_scene, increment36_scene_json, increment37_scene, increment37_scene_json, increment38_scene, increment38_scene_json, increment39_scene, increment39_scene_json, increment40_scene, increment40_scene_json, increment41_scene, increment41_scene_json, increment42_scene, increment42_scene_json, increment43_scene, increment43_scene_json, increment44_scene, increment44_scene_json, increment45_scene, increment45_scene_json, increment46_scene, increment46_scene_json, increment47_scene, increment47_scene_json, increment48_scene, increment48_scene_json, increment49_scene, increment49_scene_json, increment50_scene, increment50_scene_json, increment51_scene, increment51_scene_json, increment52_scene, increment52_scene_json, increment53_scene, increment53_scene_json, increment54_scene, increment54_scene_json, increment55_scene, increment55_scene_json, increment56_scene, increment56_scene_json, increment57_scene, increment57_scene_json, increment58_scene, increment58_scene_json, increment59_scene, increment59_scene_json, increment60_scene, increment60_scene_json, increment61_scene, increment61_scene_json, increment62_scene, increment62_scene_json, increment63_scene, increment63_scene_json, increment64_scene, increment64_scene_json, increment65_scene, increment65_scene_json, scene_catalog, catalog_ids, scene_by_id, parse_scene, Body, Camera, CameraFollow, CharacterController, CollisionGroups, DespawnEvent, DropEvent, Pickup, PlayUntil, SpawnEvent, Transition, Win, UseEvent,
    Impulse, Joint, Light, Material, MeshCollider, RayHit, Raycast, Scene, Shape, Shapecast, SweepHit, Trigger,
};

use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STEPS: u32 = 90;
pub const DEFAULT_DT: f32 = 1.0 / 60.0;
pub const INCREMENT2_STEPS: u32 = 120;
pub const INCREMENT3_FRAMES: u32 = 10;
pub const INCREMENT3_STRIDE: u32 = 20;
pub const INCREMENT4_STEPS: u32 = 100;
pub const INCREMENT5_STEPS: u32 = 100;
pub const INCREMENT6_STEPS: u32 = 100;
pub const INCREMENT7_STEPS: u32 = 100;
pub const INCREMENT8_STEPS: u32 = 100;
pub const INCREMENT9_STEPS: u32 = 100;
pub const INCREMENT10_STEPS: u32 = 100;
pub const INCREMENT11_STEPS: u32 = 100;
pub const INCREMENT12_STEPS: u32 = 100;
pub const INCREMENT12_ORBIT_FRAMES: u32 = 8;
pub const INCREMENT13_STEPS: u32 = 100;
pub const INCREMENT14_STEPS: u32 = 100;
pub const INCREMENT15_FRAMES: u32 = 8;
pub const INCREMENT15_STRIDE: u32 = 12;
pub const INCREMENT16_STEPS: u32 = 100;
pub const INCREMENT17_STEPS: u32 = 100;
pub const INCREMENT18_STEPS: u32 = 100;
pub const INCREMENT19_STEPS: u32 = 100;
pub const INCREMENT20_STEPS: u32 = 100;
pub const INCREMENT21_STEPS: u32 = 100;
pub const INCREMENT22_STEPS: u32 = 100;
pub const INCREMENT23_STEPS: u32 = 100;
pub const INCREMENT24_STEPS: u32 = 100;
pub const INCREMENT25_STEPS: u32 = 100;
pub const INCREMENT26_STEPS: u32 = 100;
pub const INCREMENT27_STEPS: u32 = 100;
pub const INCREMENT28_STEPS: u32 = 120;
pub const INCREMENT29_STEPS: u32 = 120;
pub const INCREMENT30_STEPS: u32 = 120;
pub const INCREMENT31_STEPS: u32 = 120;
pub const INCREMENT32_STEPS: u32 = 120;
pub const INCREMENT33_STEPS: u32 = 120;
pub const INCREMENT34_STEPS: u32 = 120;
pub const INCREMENT35_STEPS: u32 = 120;
pub const INCREMENT36_STEPS: u32 = 120;
pub const INCREMENT37_STEPS: u32 = 120;
pub const INCREMENT38_STEPS: u32 = 120;
pub const INCREMENT39_STEPS: u32 = 120;
pub const INCREMENT40_STEPS: u32 = 120;
pub const INCREMENT41_STEPS: u32 = 120;
pub const INCREMENT42_STEPS: u32 = 120;
pub const INCREMENT43_STEPS: u32 = 120;
pub const INCREMENT44_STEPS: u32 = 120;
pub const INCREMENT45_STEPS: u32 = 120;
pub const INCREMENT46_STEPS: u32 = 120;
pub const INCREMENT47_STEPS: u32 = 120;
pub const INCREMENT48_STEPS: u32 = 120;
pub const INCREMENT49_STEPS: u32 = 120;
pub const INCREMENT50_STEPS: u32 = 120;
pub const INCREMENT51_STEPS: u32 = 120;
pub const INCREMENT52_STEPS: u32 = 120;
pub const INCREMENT53_STEPS: u32 = 120;
pub const INCREMENT54_STEPS: u32 = 120;
pub const INCREMENT55_STEPS: u32 = 120;
pub const INCREMENT56_STEPS: u32 = 120;
pub const INCREMENT57_STEPS: u32 = 120;
pub const INCREMENT58_STEPS: u32 = 120;
pub const INCREMENT59_STEPS: u32 = 120;
pub const INCREMENT60_STEPS: u32 = 120;
pub const INCREMENT61_STEPS: u32 = 120;
pub const INCREMENT62_STEPS: u32 = 120;
pub const INCREMENT63_STEPS: u32 = 120;
pub const INCREMENT64_STEPS: u32 = 120;
pub const INCREMENT65_STEPS: u32 = 120;

#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    pub scene: PathBuf,
    pub physics: PathBuf,
    pub frame: PathBuf,
}

/// Write scene JSON, step physics, write physics dump, render post-step PNG.
pub fn run_increment1(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &demo_scene(), steps, dt, width, height)
}

/// Increment 2: metal ball / rough crate / metal stopper, step, render post-step PNG.
pub fn run_increment2(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment2_scene(), steps, dt, width, height)
}

/// Increment 4: triangle-mesh rock + primitives, step, render post-step PNG.
pub fn run_increment4(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment4_scene(), steps, dt, width, height)
}

/// Increment 5: textured mesh + primitives, step, render post-step PNG.
pub fn run_increment5(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment5_scene(), steps, dt, width, height)
}

/// Increment 6: two distinct meshes + primitives, step, render post-step PNG.
pub fn run_increment6(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment6_scene(), steps, dt, width, height)
}

/// Increment 7: environment mesh as ground + props, step, render post-step PNG.
pub fn run_increment7(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment7_scene(), steps, dt, width, height)
}

/// Increment 8: increment-7 courtyard plus a glTF pillar, step, render post-step PNG.
pub fn run_increment8(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment8_scene(), steps, dt, width, height)
}

/// Increment 9: same courtyard; pillar look from glTF pbrMetallicRoughness.
pub fn run_increment9(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment9_scene(), steps, dt, width, height)
}

/// Increment 10: increment-9 courtyard plus a local point light.
pub fn run_increment10(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment10_scene(), steps, dt, width, height)
}

/// Increment 11: same courtyard; the point light now casts shadow rays.
pub fn run_increment11(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment11_scene(), steps, dt, width, height)
}

/// Camera position on a Y-up orbit. Index i of 8 → yaw i * 45°.
pub fn orbit_camera_position(look_at: [f32; 3], radius: f32, height: f32, index: u32) -> [f32; 3] {
    let yaw = (index as f32) * (std::f32::consts::PI / 4.0);
    [
        look_at[0] + radius * yaw.sin(),
        height,
        look_at[2] + radius * yaw.cos(),
    ]
}

/// Horizontal radius (XZ from look-at) and camera height of an authored camera.
pub fn orbit_radius_and_height(camera: &Camera) -> (f32, f32) {
    let dx = camera.position[0] - camera.look_at[0];
    let dz = camera.position[2] - camera.look_at[2];
    ((dx * dx + dz * dz).sqrt(), camera.position[1])
}

#[derive(Debug, Clone)]
pub struct Increment12Paths {
    pub scene: PathBuf,
    pub physics: PathBuf,
    pub frames: Vec<PathBuf>,
}

/// Increment 12: step the increment-11 courtyard once, then render 8 orbit cameras.
pub fn run_increment12(
    out_dir: &Path,
    steps: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<Increment12Paths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;

    let scene = increment12_scene();
    let scene_path = out_dir.join("scene.json");
    let physics_path = out_dir.join("physics.json");

    let json = serde_json::to_string_pretty(&scene).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;

    let dump = step_physics(&scene, steps, dt)?;
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(&physics_path, dump_json).map_err(|e| format!("write physics: {e}"))?;

    let mut framed = scene.clone();
    apply_physics_to_scene(&mut framed, &dump);

    let (radius, cam_height) = orbit_radius_and_height(&scene.camera);
    let look_at = scene.camera.look_at;
    let mut frames = Vec::with_capacity(INCREMENT12_ORBIT_FRAMES as usize);
    for i in 0..INCREMENT12_ORBIT_FRAMES {
        let mut view = framed.clone();
        view.camera.position = orbit_camera_position(look_at, radius, cam_height, i);
        let path = out_dir.join(format!("frame_{i:02}.png"));
        render_scene_to_png(&view, width, height, &path)?;
        frames.push(path);
    }

    Ok(Increment12Paths {
        scene: scene_path,
        physics: physics_path,
        frames,
    })
}

/// Increment 13: same courtyard; glTF metallicRoughnessTexture drives per-texel MR.
pub fn run_increment13(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment13_scene(), steps, dt, width, height)
}

/// Increment 14: same courtyard; glTF normalTexture drives tangent-space bump.
pub fn run_increment14(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment14_scene(), steps, dt, width, height)
}

/// Increment 16: same courtyard still; glTF emissiveFactor * emissiveTexture added after lighting.
pub fn run_increment16(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment16_scene(), steps, dt, width, height)
}

/// Increment 17: increment-16 courtyard plus a BLEND glass pane; ray continues and composites.
pub fn run_increment17(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment17_scene(), steps, dt, width, height)
}

/// Increment 18: increment-17 courtyard; warm lamp is a rectangular area light (soft penumbra).
pub fn run_increment18(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment18_scene(), steps, dt, width, height)
}

/// Increment 19: increment-18 courtyard; glTF occlusionTexture multiplies IBL/ambient on the pillar.
pub fn run_increment19(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment19_scene(), steps, dt, width, height)
}

/// Increment 20: increment-19 courtyard; pane transmits and the ray refracts (Snell, authored IOR).
pub fn run_increment20(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment20_scene(), steps, dt, width, height)
}

/// Increment 21: increment-20 courtyard plus two authored mesh bodies (crate + bench).
pub fn run_increment21(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment21_scene(), steps, dt, width, height)
}

/// Increment 22: increment-21 courtyard; gold ball has authored clearcoat + clearcoat_roughness.
pub fn run_increment22(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment22_scene(), steps, dt, width, height)
}

/// Increment 23: increment-22 courtyard; bench has authored sheen + sheen_roughness + sheen_color.
pub fn run_increment23(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment23_scene(), steps, dt, width, height)
}

/// Increment 24: increment-23 courtyard; pane has KHR_materials_volume Beer-Lambert attenuation.
pub fn run_increment24(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment24_scene(), steps, dt, width, height)
}

/// Increment 25: increment-24 courtyard; gold ball has authored anisotropy + anisotropy_rotation.
pub fn run_increment25(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment25_scene(), steps, dt, width, height)
}

/// Increment 26: increment-25 courtyard; gold ball has authored thin-film iridescence.
pub fn run_increment26(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment26_scene(), steps, dt, width, height)
}

/// Increment 27: increment-26 courtyard; pane has authored KHR_materials_dispersion.
pub fn run_increment27(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment27_scene(), steps, dt, width, height)
}

/// Increment 28: increment-27 courtyard plus a hanging lantern on the pillar (Rapier hinge).
pub fn run_increment28(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment28_scene(), steps, dt, width, height)
}

/// Increment 29: increment-28 courtyard plus a drawer on the crate (Rapier slider).
pub fn run_increment29(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment29_scene(), steps, dt, width, height)
}

/// Increment 30: increment-29 courtyard; lantern hinge has an authored motor.
pub fn run_increment30(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment30_scene(), steps, dt, width, height)
}

/// Increment 31: increment-30 courtyard plus a charm on a ball socket.
pub fn run_increment31(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment31_scene(), steps, dt, width, height)
}

/// Increment 32: increment-31 courtyard plus a drawer-open sensor volume.
pub fn increment32() -> crate::scene::Scene {
    increment32_scene()
}

/// Increment 32: increment-31 courtyard plus a drawer-open sensor volume.
pub fn run_increment32(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment32_scene(), steps, dt, width, height)
}

/// Increment 33: increment-32 courtyard; existing lantern is an emissive mesh light.
pub fn increment33() -> crate::scene::Scene {
    increment33_scene()
}

/// Increment 33: increment-32 courtyard; existing lantern is an emissive mesh light.
pub fn run_increment33(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment33_scene(), steps, dt, width, height)
}

/// Increment 34: increment-33 courtyard; existing crate–drawer slider has a motor.
pub fn increment34() -> crate::scene::Scene {
    increment34_scene()
}

/// Increment 34: increment-33 courtyard; existing crate–drawer slider has a motor.
pub fn run_increment34(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment34_scene(), steps, dt, width, height)
}

/// Increment 35: increment-34 courtyard plus an authored drawer_probe raycast.
pub fn increment35() -> crate::scene::Scene {
    increment35_scene()
}

/// Increment 35: increment-34 courtyard plus an authored drawer_probe raycast.
pub fn run_increment35(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment35_scene(), steps, dt, width, height)
}

/// Increment 36: increment-35 courtyard plus an authored drawer_sweep shapecast.
pub fn increment36() -> crate::scene::Scene {
    increment36_scene()
}

/// Increment 36: increment-35 courtyard plus an authored drawer_sweep shapecast.
pub fn run_increment36(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment36_scene(), steps, dt, width, height)
}

/// Increment 37: increment-36 courtyard plus a crate lid on a fixed joint.
pub fn increment37() -> crate::scene::Scene {
    increment37_scene()
}

/// Increment 37: increment-36 courtyard plus a crate lid on a fixed joint.
pub fn run_increment37(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment37_scene(), steps, dt, width, height)
}

/// Increment 38: increment-37 courtyard plus an authored impulse on the gold ball.
pub fn increment38() -> crate::scene::Scene {
    increment38_scene()
}

/// Increment 38: increment-37 courtyard plus an authored impulse on the gold ball.
pub fn run_increment38(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment38_scene(), steps, dt, width, height)
}

/// Increment 39: increment-38 courtyard plus a kinematic moving platform.
pub fn increment39() -> crate::scene::Scene {
    increment39_scene()
}

/// Increment 39: increment-38 courtyard plus a kinematic moving platform.
pub fn run_increment39(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment39_scene(), steps, dt, width, height)
}

/// Increment 40: increment-39 courtyard plus a dynamic rider on the platform.
pub fn increment40() -> crate::scene::Scene {
    increment40_scene()
}

/// Increment 40: increment-39 courtyard plus a dynamic rider on the platform.
pub fn run_increment40(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment40_scene(), steps, dt, width, height)
}

/// Increment 41: increment-40 courtyard plus a limited hinge gate.
pub fn increment41() -> crate::scene::Scene {
    increment41_scene()
}

/// Increment 41: increment-40 courtyard plus a limited hinge gate.
pub fn run_increment41(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment41_scene(), steps, dt, width, height)
}

/// Increment 42: increment-41 courtyard plus a brass bob on a distance/rope joint.
pub fn increment42() -> crate::scene::Scene {
    increment42_scene()
}

/// Increment 42: increment-41 courtyard plus a brass bob on a distance/rope joint.
pub fn run_increment42(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment42_scene(), steps, dt, width, height)
}

/// Increment 43: increment-42 courtyard; the gate–bob rope is breakable.
pub fn increment43() -> crate::scene::Scene {
    increment43_scene()
}

/// Increment 43: increment-42 courtyard; the gate–bob rope is breakable.
pub fn run_increment43(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment43_scene(), steps, dt, width, height)
}

/// Increment 44: increment-43 courtyard plus a cork hung from the gate on a spring.
pub fn increment44() -> crate::scene::Scene {
    increment44_scene()
}

/// Increment 44: increment-43 courtyard plus a cork hung from the gate on a spring.
pub fn run_increment44(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment44_scene(), steps, dt, width, height)
}

/// Increment 45: increment-44 courtyard; gate hinge uses a position motor to 0.55.
pub fn increment45() -> crate::scene::Scene {
    increment45_scene()
}

/// Increment 45: increment-44 courtyard; gate hinge uses a position motor to 0.55.
pub fn run_increment45(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment45_scene(), steps, dt, width, height)
}

/// Increment 46: increment-45 courtyard; single camera re-aimed at the gate cluster.
pub fn increment46() -> crate::scene::Scene {
    increment46_scene()
}

/// Increment 46: increment-45 courtyard; single camera re-aimed at the gate cluster.
pub fn run_increment46(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment46_scene(), steps, dt, width, height)
}

/// Increment 47: increment-46 courtyard; physics dump records contact events.
pub fn increment47() -> crate::scene::Scene {
    increment47_scene()
}

/// Increment 47: increment-46 courtyard; physics dump records contact events.
pub fn run_increment47(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment47_scene(), steps, dt, width, height)
}

/// Increment 48: increment-47 courtyard plus an authorable Rapier walker.
pub fn increment48() -> crate::scene::Scene {
    increment48_scene()
}

/// Increment 48: increment-47 courtyard plus an authorable Rapier walker.
pub fn run_increment48(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment48_scene(), steps, dt, width, height)
}

/// Increment 49: increment-48 courtyard plus authorable Rapier collision groups.
pub fn increment49() -> crate::scene::Scene {
    increment49_scene()
}

/// Increment 49: increment-48 courtyard plus a yellow bar the walker ignores.
pub fn run_increment49(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment49_scene(), steps, dt, width, height)
}

/// Increment 50: increment-49 courtyard plus authorable spawn/despawn.
pub fn increment50() -> crate::scene::Scene {
    increment50_scene()
}

/// Increment 50: gold token appears at 30; yellow bar leaves at 80.
pub fn run_increment50(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment50_scene(), steps, dt, width, height)
}

/// Increment 51: increment-50 courtyard plus authorable pickup-on-overlap.
pub fn increment51() -> crate::scene::Scene {
    increment51_scene()
}

/// Increment 51: walker picks up the gold token in token_zone.
pub fn run_increment51(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment51_scene(), steps, dt, width, height)
}

/// Increment 52: increment-51 courtyard plus authorable follow-cam.
pub fn increment52() -> crate::scene::Scene {
    increment52_scene()
}

/// Increment 52: camera follows the walker.
pub fn run_increment52(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment52_scene(), steps, dt, width, height)
}

/// Increment 53: increment-52 courtyard plus authorable play-until.
pub fn increment53() -> crate::scene::Scene {
    increment53_scene()
}

/// Increment 53: run until the token is picked (steps is a max cap).
pub fn run_increment53(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment53_scene(), steps, dt, width, height)
}

/// Increment 54: a second scene, a short stone lane.
pub fn increment54() -> crate::scene::Scene {
    increment54_scene()
}

/// Increment 54: walker walks +x to a gold token; play-until stops on pickup.
pub fn run_increment54(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment54_scene(), steps, dt, width, height)
}

/// Increment 55: increment-54 lane plus play-until entered.
pub fn increment55() -> crate::scene::Scene {
    increment55_scene()
}

/// Increment 55: walker picks up the token then stops on the exit pad.
pub fn run_increment55(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment55_scene(), steps, dt, width, height)
}

/// Increment 56: increment-55 lane plus hold-on-pickup.
pub fn increment56() -> crate::scene::Scene {
    increment56_scene()
}

/// Increment 56: walker carries the token to the exit (steps is a max cap).
pub fn run_increment56(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment56_scene(), steps, dt, width, height)
}

/// Increment 57: increment-56 lane plus authorable drop.
pub fn increment57() -> crate::scene::Scene {
    increment57_scene()
}

/// Increment 57: walker drops the token at the exit (steps is a max cap).
pub fn run_increment57(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment57_scene(), steps, dt, width, height)
}

/// Increment 58: increment-57 lane plus a second walker / NPC.
pub fn increment58() -> crate::scene::Scene {
    increment58_scene()
}

/// Increment 58: amber npc walks -x and passes the coral walker (steps is a max cap).
pub fn run_increment58(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    write_step_render(out_dir, &increment58_scene(), steps, dt, width, height)
}

/// Increment 59: increment-58 lane with Scene.id = "lane".
pub fn increment59() -> crate::scene::Scene {
    increment59_scene()
}

/// Write `[{id:courtyard},{id:lane}]` for the named-scene catalog.
pub fn write_scenes_catalog(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create {parent:?}: {e}"))?;
        }
    }
    let catalog = serde_json::json!([
        { "id": "courtyard" },
        { "id": "lane" }
    ]);
    let json = serde_json::to_string_pretty(&catalog).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("write scenes catalog: {e}"))?;
    Ok(())
}

/// Increment 59: named lane scene (steps is a max cap). Also writes scenes.json.
pub fn run_increment59(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment59_scene(), steps, dt, width, height)?;
    write_scenes_catalog(&out_dir.join("scenes.json"))?;
    Ok(paths)
}

/// Increment 60: catalog courtyard (no Scene.id; dump key applied after step).
pub fn increment60() -> crate::scene::Scene {
    increment60_scene()
}

/// Look up a catalog scene, step it, stamp dump.scene with the catalog key
/// when the authored scene has no id (courtyard). increment53 stays
/// scene-key-free because it still calls step_physics / write_step_render.
pub fn run_catalog_scene(
    id: &str,
    out_dir: &Path,
    steps: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<ArtifactPaths, String> {
    run_catalog_scene_with_carry(id, out_dir, steps, dt, width, height, None)
}

/// Run a catalog scene, optionally injecting held bodies from a prior dump.
pub fn run_catalog_scene_with_carry(
    id: &str,
    out_dir: &Path,
    steps: u32,
    dt: f32,
    width: u32,
    height: u32,
    carry: Option<&PhysicsDump>,
) -> Result<ArtifactPaths, String> {
    let scene = scene_by_id(id).ok_or_else(|| format!("unknown scene id: {id}"))?;
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;

    let scene_path = out_dir.join("scene.json");
    let physics_path = out_dir.join("physics.json");
    let frame_path = out_dir.join("frame.png");

    let json = serde_json::to_string_pretty(&scene).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;

    let dump = step_catalog_scene_with_carry(id, steps, dt, carry)?;
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(&physics_path, dump_json).map_err(|e| format!("write physics: {e}"))?;

    let mut framed = scene.clone();
    apply_physics_to_scene(&mut framed, &dump);
    render_scene_to_png(&framed, width, height, &frame_path)?;

    Ok(ArtifactPaths {
        scene: scene_path,
        physics: physics_path,
        frame: frame_path,
    })
}

/// Increment 60: run the catalog courtyard (sim --scene courtyard).
pub fn run_increment60(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    run_catalog_scene("courtyard", out_dir, steps, dt, width, height)
}

/// Increment 61: increment-59 lane plus authorable transition to courtyard.
pub fn increment61() -> crate::scene::Scene {
    increment61_scene()
}

/// Increment 61: lane authors transition to courtyard. After play-until
/// fires, dump.transition stamps { to, at_step }. Then if that `to` is
/// set, run the catalog scene into the same dir as next-physics.json
/// + next-frame.png (do not overwrite frame.png / physics.json).
pub fn run_increment61(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment61_scene(), steps, dt, width, height)?;
    let dump: PhysicsDump = {
        let txt = fs::read_to_string(&paths.physics).map_err(|e| format!("read physics: {e}"))?;
        serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?
    };
    if let Some(tr) = &dump.transition {
        let next_scene = scene_by_id(&tr.to)
            .ok_or_else(|| format!("unknown transition target: {}", tr.to))?;
        let next_dump = step_catalog_scene(&tr.to, steps, dt)?;
        let next_scene_path = out_dir.join("next-scene.json");
        let next_physics = out_dir.join("next-physics.json");
        let next_frame = out_dir.join("next-frame.png");
        let scene_json = serde_json::to_string_pretty(&next_scene).map_err(|e| e.to_string())?;
        fs::write(&next_scene_path, scene_json).map_err(|e| format!("write next-scene: {e}"))?;
        let dump_json = serde_json::to_string_pretty(&next_dump).map_err(|e| e.to_string())?;
        fs::write(&next_physics, dump_json).map_err(|e| format!("write next-physics: {e}"))?;
        let mut framed = next_scene.clone();
        apply_physics_to_scene(&mut framed, &next_dump);
        render_scene_to_png(&framed, width, height, &next_frame)?;
    }
    Ok(paths)
}


/// Increment 62: increment-61 lane without the drop. Held items survive
/// the scene change.
pub fn increment62() -> crate::scene::Scene {
    increment62_scene()
}

/// Increment 62: write the no-drop lane, then if dump.transition.to is
/// set, run sim --scene <to> --carry physics.json into the same dir as
/// next-physics.json + next-frame.png (do not overwrite the lane files).
pub fn run_increment62(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment62_scene(), steps, dt, width, height)?;
    let dump: PhysicsDump = {
        let txt = fs::read_to_string(&paths.physics).map_err(|e| format!("read physics: {e}"))?;
        serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?
    };
    if let Some(tr) = &dump.transition {
        let next_scene = scene_by_id(&tr.to)
            .ok_or_else(|| format!("unknown transition target: {}", tr.to))?;
        let next_dump = step_catalog_scene_with_carry(&tr.to, steps, dt, Some(&dump))?;
        let next_scene_path = out_dir.join("next-scene.json");
        let next_physics = out_dir.join("next-physics.json");
        let next_frame = out_dir.join("next-frame.png");
        let scene_json = serde_json::to_string_pretty(&next_scene).map_err(|e| e.to_string())?;
        fs::write(&next_scene_path, scene_json).map_err(|e| format!("write next-scene: {e}"))?;
        let dump_json = serde_json::to_string_pretty(&next_dump).map_err(|e| e.to_string())?;
        fs::write(&next_physics, dump_json).map_err(|e| format!("write next-physics: {e}"))?;
        let mut framed = next_scene.clone();
        apply_physics_to_scene(&mut framed, &next_dump);
        render_scene_to_png(&framed, width, height, &next_frame)?;
    }
    Ok(paths)
}

/// Increment 63: increment-62 lane plus an authorable win.
pub fn increment63() -> crate::scene::Scene {
    increment63_scene()
}

/// Increment 63: write the win-authored lane, then if dump.transition.to
/// is set, carry into the destination and stamp dump.won on next-physics
/// when held still includes the win body. Does not mutate run_increment62.
pub fn run_increment63(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment63_scene(), steps, dt, width, height)?;
    let dump: PhysicsDump = {
        let txt = fs::read_to_string(&paths.physics).map_err(|e| format!("read physics: {e}"))?;
        serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?
    };
    if let Some(tr) = &dump.transition {
        let next_scene = scene_by_id(&tr.to)
            .ok_or_else(|| format!("unknown transition target: {}", tr.to))?;
        let mut next_dump = step_catalog_scene_with_carry(&tr.to, steps, dt, Some(&dump))?;
        apply_win(&mut next_dump, increment63_scene().win.as_ref());
        let next_scene_path = out_dir.join("next-scene.json");
        let next_physics = out_dir.join("next-physics.json");
        let next_frame = out_dir.join("next-frame.png");
        let scene_json = serde_json::to_string_pretty(&next_scene).map_err(|e| e.to_string())?;
        fs::write(&next_scene_path, scene_json).map_err(|e| format!("write next-scene: {e}"))?;
        let dump_json = serde_json::to_string_pretty(&next_dump).map_err(|e| e.to_string())?;
        fs::write(&next_physics, dump_json).map_err(|e| format!("write next-physics: {e}"))?;
        let mut framed = next_scene.clone();
        apply_physics_to_scene(&mut framed, &next_dump);
        render_scene_to_png(&framed, width, height, &next_frame)?;
    }
    Ok(paths)
}

/// Increment 64: increment-63 lane with the increment61 drop restored.
pub fn increment64() -> crate::scene::Scene {
    increment64_scene()
}

/// Increment 64: write the drop+win lane, then if dump.transition.to
/// is set, carry into the destination and apply the outcome. Token is
/// dropped on exit so courtyard stamps dump.lost empty_handed.
/// Does not mutate run_increment63.
pub fn run_increment64(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment64_scene(), steps, dt, width, height)?;
    let dump: PhysicsDump = {
        let txt = fs::read_to_string(&paths.physics).map_err(|e| format!("read physics: {e}"))?;
        serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?
    };
    if let Some(tr) = &dump.transition {
        let next_scene = scene_by_id(&tr.to)
            .ok_or_else(|| format!("unknown transition target: {}", tr.to))?;
        let mut next_dump = step_catalog_scene_with_carry(&tr.to, steps, dt, Some(&dump))?;
        apply_win(&mut next_dump, increment64_scene().win.as_ref());
        let next_scene_path = out_dir.join("next-scene.json");
        let next_physics = out_dir.join("next-physics.json");
        let next_frame = out_dir.join("next-frame.png");
        let scene_json = serde_json::to_string_pretty(&next_scene).map_err(|e| e.to_string())?;
        fs::write(&next_scene_path, scene_json).map_err(|e| format!("write next-scene: {e}"))?;
        let dump_json = serde_json::to_string_pretty(&next_dump).map_err(|e| e.to_string())?;
        fs::write(&next_physics, dump_json).map_err(|e| format!("write next-physics: {e}"))?;
        let mut framed = next_scene.clone();
        apply_physics_to_scene(&mut framed, &next_dump);
        render_scene_to_png(&framed, width, height, &next_frame)?;
    }
    Ok(paths)
}

/// Increment 65: increment-63 lane plus an authorable use.
pub fn increment65() -> crate::scene::Scene {
    increment65_scene()
}

/// Increment 65: write the use+win lane, then if dump.transition.to
/// is set, carry into the destination and apply_win (token still
/// held → won). Does not mutate run_increment63 or run_increment64.
pub fn run_increment65(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    let paths = write_step_render(out_dir, &increment65_scene(), steps, dt, width, height)?;
    let dump: PhysicsDump = {
        let txt = fs::read_to_string(&paths.physics).map_err(|e| format!("read physics: {e}"))?;
        serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?
    };
    if let Some(tr) = &dump.transition {
        let next_scene = scene_by_id(&tr.to)
            .ok_or_else(|| format!("unknown transition target: {}", tr.to))?;
        let mut next_dump = step_catalog_scene_with_carry(&tr.to, steps, dt, Some(&dump))?;
        apply_win(&mut next_dump, increment65_scene().win.as_ref());
        let next_scene_path = out_dir.join("next-scene.json");
        let next_physics = out_dir.join("next-physics.json");
        let next_frame = out_dir.join("next-frame.png");
        let scene_json = serde_json::to_string_pretty(&next_scene).map_err(|e| e.to_string())?;
        fs::write(&next_scene_path, scene_json).map_err(|e| format!("write next-scene: {e}"))?;
        let dump_json = serde_json::to_string_pretty(&next_dump).map_err(|e| e.to_string())?;
        fs::write(&next_physics, dump_json).map_err(|e| format!("write next-physics: {e}"))?;
        let mut framed = next_scene.clone();
        apply_physics_to_scene(&mut framed, &next_dump);
        render_scene_to_png(&framed, width, height, &next_frame)?;
    }
    Ok(paths)
}


#[derive(Debug, Clone)]
pub struct Increment15Paths {
    pub scene: PathBuf,
    pub physics: PathBuf,
    pub frames: Vec<PathBuf>,
}

/// Increment 15: increment-14 courtyard, shove the ball, 8 physics frames, one camera.
pub fn run_increment15(
    out_dir: &Path,
    frames: u32,
    frame_stride: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<Increment15Paths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;

    let scene = increment15_scene();
    let scene_path = out_dir.join("scene.json");
    let physics_path = out_dir.join("physics.json");

    let json = serde_json::to_string_pretty(&scene).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;

    let traj = simulate_trajectory(&scene, frames, frame_stride, dt)?;
    let last = traj
        .frames
        .last()
        .ok_or_else(|| "trajectory produced no frames".to_string())?;
    let dump = serde_json::json!({
        "steps": last.step,
        "dt": traj.dt,
        "gravity": [0.0, -9.81, 0.0],
        "bodies": last.bodies,
        "contacts": last.contacts,
        "frame_stride": traj.frame_stride,
        "frames": traj.frames,
    });
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(&physics_path, dump_json).map_err(|e| format!("write physics: {e}"))?;

    let mut frame_paths = Vec::with_capacity(traj.frames.len());
    for (i, snap) in traj.frames.iter().enumerate() {
        let mut framed = scene.clone();
        apply_body_states(&mut framed, &snap.bodies);
        let path = out_dir.join(format!("frame_{i:02}.png"));
        render_scene_to_png(&framed, width, height, &path)?;
        frame_paths.push(path);
    }

    Ok(Increment15Paths {
        scene: scene_path,
        physics: physics_path,
        frames: frame_paths,
    })
}

fn write_step_render(
    out_dir: &Path,
    scene: &Scene,
    steps: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<ArtifactPaths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;

    let scene_path = out_dir.join("scene.json");
    let physics_path = out_dir.join("physics.json");
    let frame_path = out_dir.join("frame.png");

    let json = serde_json::to_string_pretty(scene).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;

    let dump = step_physics(scene, steps, dt)?;
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(&physics_path, dump_json).map_err(|e| format!("write physics: {e}"))?;

    let mut framed = scene.clone();
    apply_physics_to_scene(&mut framed, &dump);
    render_scene_to_png(&framed, width, height, &frame_path)?;

    Ok(ArtifactPaths {
        scene: scene_path,
        physics: physics_path,
        frame: frame_path,
    })
}

pub fn apply_physics_to_scene(scene: &mut Scene, dump: &PhysicsDump) {
    let live: std::collections::HashSet<&str> = dump.bodies.iter().map(|b| b.id.as_str()).collect();
    scene.bodies.retain(|b| live.contains(b.id.as_str()));
    for spawn in &scene.spawns {
        if live.contains(spawn.body.id.as_str())
            && !scene.bodies.iter().any(|b| b.id == spawn.body.id)
        {
            scene.bodies.push(spawn.body.clone());
        }
    }
    apply_body_states(scene, &dump.bodies);
    scene.ray_hits = dump.ray_hits.clone();
    scene.sweep_hits = dump.sweep_hits.clone();
    if let Some(cam) = &dump.camera {
        scene.camera.position = cam.position;
        scene.camera.look_at = cam.look_at;
    }
}

pub fn apply_body_states(scene: &mut Scene, bodies: &[PhysicsBodyState]) {
    for body in &mut scene.bodies {
        if let Some(state) = bodies.iter().find(|b| b.id == body.id) {
            body.position = state.position;
            body.rotation_wxyz = state.rotation_wxyz;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimPaths {
    pub trajectory: PathBuf,
    pub frame: PathBuf,
    pub frames_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Increment3Paths {
    pub scene: PathBuf,
    pub trajectory: PathBuf,
    pub frame: PathBuf,
    pub frames_dir: PathBuf,
}

/// Write authored increment-3 scene, then simulate and render frames.
pub fn run_increment3(
    out_dir: &Path,
    frames: u32,
    frame_stride: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<Increment3Paths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;
    let scene_path = out_dir.join("scene.json");
    let json = serde_json::to_string_pretty(&increment3_scene()).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;
    let sim = sim_scene(&increment3_scene(), out_dir, frames, frame_stride, dt, width, height)?;
    Ok(Increment3Paths {
        scene: scene_path,
        trajectory: sim.trajectory,
        frame: sim.frame,
        frames_dir: sim.frames_dir,
    })
}

/// Step a scene over time, write trajectory.json + frames/frame_XX.png + last frame.png.
pub fn sim_scene(
    scene: &Scene,
    out_dir: &Path,
    frames: u32,
    frame_stride: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<SimPaths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;
    let frames_dir = out_dir.join("frames");
    fs::create_dir_all(&frames_dir).map_err(|e| format!("create {frames_dir:?}: {e}"))?;

    let traj = simulate_trajectory(scene, frames, frame_stride, dt)?;
    let trajectory_path = out_dir.join("trajectory.json");
    let traj_json = serde_json::to_string_pretty(&traj).map_err(|e| e.to_string())?;
    fs::write(&trajectory_path, traj_json).map_err(|e| format!("write trajectory: {e}"))?;

    let mut last_frame = out_dir.join("frame.png");
    for (i, snap) in traj.frames.iter().enumerate() {
        let mut framed = scene.clone();
        apply_body_states(&mut framed, &snap.bodies);
        let name = format!("frame_{i:02}.png");
        let path = frames_dir.join(&name);
        render_scene_to_png(&framed, width, height, &path)?;
        if i + 1 == traj.frames.len() {
            last_frame = out_dir.join("frame.png");
            fs::copy(&path, &last_frame).map_err(|e| format!("copy last frame: {e}"))?;
        }
    }

    Ok(SimPaths {
        trajectory: trajectory_path,
        frame: last_frame,
        frames_dir,
    })
}

pub fn sim_scene_file(
    scene_path: &Path,
    out_dir: &Path,
    frames: u32,
    frame_stride: u32,
    dt: f32,
    width: u32,
    height: u32,
) -> Result<SimPaths, String> {
    let scene = load_scene_file(scene_path)?;
    sim_scene(&scene, out_dir, frames, frame_stride, dt, width, height)
}

pub fn load_scene_file(path: &Path) -> Result<Scene, String> {
    crate::scene::load_scene_from_path(path)
}

pub fn step_scene_file(scene_path: &Path, out_path: &Path, steps: u32, dt: f32) -> Result<(), String> {
    let scene = load_scene_file(scene_path)?;
    let dump = step_physics(&scene, steps, dt)?;
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create {parent:?}: {e}"))?;
        }
    }
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(out_path, dump_json).map_err(|e| format!("write physics: {e}"))?;
    Ok(())
}

pub fn render_scene_file(
    scene_path: &Path,
    out_path: &Path,
    physics_path: Option<&Path>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut scene = load_scene_file(scene_path)?;
    if let Some(pp) = physics_path {
        let txt = fs::read_to_string(pp).map_err(|e| format!("read physics {pp:?}: {e}"))?;
        let dump: PhysicsDump = serde_json::from_str(&txt).map_err(|e| format!("parse physics: {e}"))?;
        apply_physics_to_scene(&mut scene, &dump);
    }
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create {parent:?}: {e}"))?;
        }
    }
    render_scene_to_png(&scene, width, height, out_path)
}
