//! Agent-native scene + physics inspect + headless PNG (increments 1–35).

mod mesh;
mod physics;
mod render;
mod scene;

pub use physics::{simulate_trajectory, step_physics, PhysicsBodyState, PhysicsContact, PhysicsDump, PhysicsOverlap, PhysicsJoint, Trajectory, TrajectoryFrame};
pub use render::{area_light_visibility, point_light_occluded, render_scene, render_scene_to_png, FRAME_HEIGHT, FRAME_WIDTH};
pub use mesh::{
    apply_tbn, load_gltf, load_mesh, load_obj, parse_obj, tbn_from_positions_uvs, GltfAlphaMode,
    GltfPbrMaterial, TriangleMesh,
};
pub use scene::{
    demo_scene, demo_scene_json, increment2_scene, increment2_scene_json, increment3_scene,
    increment3_scene_json, increment4_scene, increment4_scene_json, increment5_scene,
    increment5_scene_json, increment6_scene, increment6_scene_json, increment7_scene, increment7_scene_json, increment8_scene, increment8_scene_json, increment9_scene, increment9_scene_json, increment10_scene, increment10_scene_json, increment11_scene, increment11_scene_json, increment12_scene, increment12_scene_json, increment13_scene, increment13_scene_json, increment14_scene, increment14_scene_json, increment15_scene, increment15_scene_json, increment16_scene, increment16_scene_json, increment17_scene, increment17_scene_json, increment18_scene, increment18_scene_json, increment19_scene, increment19_scene_json, increment20_scene, increment20_scene_json, increment21_scene, increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene, increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene, increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene, increment27_scene_json, increment28_scene, increment28_scene_json, increment29_scene, increment29_scene_json, increment30_scene, increment30_scene_json, increment31_scene, increment31_scene_json, increment32_scene, increment32_scene_json, increment33_scene, increment33_scene_json, increment34_scene, increment34_scene_json, increment35_scene, increment35_scene_json, parse_scene, Body, Camera,
    Joint, Light, Material, MeshCollider, RayHit, Raycast, Scene, Shape, Trigger,
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
    apply_body_states(scene, &dump.bodies);
    scene.ray_hits = dump.ray_hits.clone();
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
