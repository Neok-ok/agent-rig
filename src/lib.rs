//! Agent-native scene + physics inspect + headless PNG (increments 1–2).

mod physics;
mod render;
mod scene;

pub use physics::{step_physics, PhysicsBodyState, PhysicsContact, PhysicsDump};
pub use render::{render_scene, render_scene_to_png, FRAME_HEIGHT, FRAME_WIDTH};
pub use scene::{
    demo_scene, demo_scene_json, increment2_scene, increment2_scene_json, parse_scene, Body, Camera,
    Light, Material, Scene, Shape,
};

use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STEPS: u32 = 90;
pub const DEFAULT_DT: f32 = 1.0 / 60.0;
pub const INCREMENT2_STEPS: u32 = 120;

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
    for body in &mut scene.bodies {
        if let Some(state) = dump.bodies.iter().find(|b| b.id == body.id) {
            body.position = state.position;
            body.rotation_wxyz = state.rotation_wxyz;
        }
    }
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
