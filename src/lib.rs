//! Agent-native scene + physics inspect + headless PNG (increments 1–3).

mod physics;
mod render;
mod scene;

pub use physics::{simulate_trajectory, step_physics, PhysicsBodyState, PhysicsContact, PhysicsDump, Trajectory, TrajectoryFrame};
pub use render::{render_scene, render_scene_to_png, FRAME_HEIGHT, FRAME_WIDTH};
pub use scene::{
    demo_scene, demo_scene_json, increment2_scene, increment2_scene_json, increment3_scene,
    increment3_scene_json, parse_scene, Body, Camera, Light, Material, Scene, Shape,
};

use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STEPS: u32 = 90;
pub const DEFAULT_DT: f32 = 1.0 / 60.0;
pub const INCREMENT2_STEPS: u32 = 120;
pub const INCREMENT3_FRAMES: u32 = 10;
pub const INCREMENT3_STRIDE: u32 = 20;

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
    let txt = fs::read_to_string(path).map_err(|e| format!("read scene {path:?}: {e}"))?;
    parse_scene(&txt)
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
