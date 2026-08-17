use std::path::PathBuf;

use agent_rig::{
    render_scene_file, run_increment1, run_increment2, run_increment3, run_increment4,
    run_increment5, run_increment6, run_increment7, run_increment8, run_increment9, run_increment10, run_increment11, run_increment12, run_increment13, run_increment14, run_increment15, run_increment16, run_increment17, run_increment18, run_increment19, run_increment20, run_increment21, run_increment22, run_increment23, run_increment24, run_increment25, run_increment26, run_increment27, run_increment28, run_increment29, run_increment30, run_increment31, run_increment32, run_increment33, run_increment34, run_increment35, run_increment36, run_increment37, run_increment38, run_increment39, run_increment40, run_increment41, run_increment42, run_increment43, run_increment44, run_increment45, run_increment46, run_increment47, run_increment48, run_increment49, run_increment50, run_increment51, run_increment52, run_increment53, run_increment54, run_increment55, run_increment56, run_increment57, run_increment58, run_increment59, run_increment60, run_increment61, run_increment62, run_increment63, run_catalog_scene_with_carry, PhysicsDump, write_scenes_catalog, catalog_ids, sim_scene_file, step_scene_file, DEFAULT_DT,
    DEFAULT_STEPS, FRAME_HEIGHT, FRAME_WIDTH, INCREMENT2_STEPS, INCREMENT3_FRAMES, INCREMENT3_STRIDE,
    INCREMENT4_STEPS, INCREMENT5_STEPS, INCREMENT6_STEPS, INCREMENT7_STEPS, INCREMENT8_STEPS,
    INCREMENT9_STEPS, INCREMENT10_STEPS, INCREMENT11_STEPS, INCREMENT12_STEPS, INCREMENT13_STEPS,
    INCREMENT14_STEPS, INCREMENT15_FRAMES, INCREMENT15_STRIDE, INCREMENT16_STEPS, INCREMENT17_STEPS, INCREMENT18_STEPS, INCREMENT19_STEPS, INCREMENT20_STEPS, INCREMENT21_STEPS, INCREMENT22_STEPS, INCREMENT23_STEPS, INCREMENT24_STEPS, INCREMENT25_STEPS, INCREMENT26_STEPS, INCREMENT27_STEPS, INCREMENT28_STEPS, INCREMENT29_STEPS, INCREMENT30_STEPS, INCREMENT31_STEPS, INCREMENT32_STEPS, INCREMENT33_STEPS, INCREMENT34_STEPS, INCREMENT35_STEPS, INCREMENT36_STEPS, INCREMENT37_STEPS, INCREMENT38_STEPS, INCREMENT39_STEPS, INCREMENT40_STEPS, INCREMENT41_STEPS, INCREMENT42_STEPS, INCREMENT43_STEPS, INCREMENT44_STEPS, INCREMENT45_STEPS, INCREMENT46_STEPS, INCREMENT47_STEPS, INCREMENT48_STEPS, INCREMENT49_STEPS, INCREMENT50_STEPS, INCREMENT51_STEPS, INCREMENT52_STEPS, INCREMENT53_STEPS, INCREMENT54_STEPS, INCREMENT55_STEPS, INCREMENT56_STEPS, INCREMENT57_STEPS, INCREMENT58_STEPS, INCREMENT59_STEPS, INCREMENT60_STEPS, INCREMENT61_STEPS, INCREMENT62_STEPS, INCREMENT63_STEPS,
};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-rig", about = "Scene JSON + physics dump + headless PNG")]
struct Args {
    /// Use the built-in increment-1 demo scene (alias for `demo`).
    #[arg(long)]
    demo: bool,
    /// Output directory for scene.json, physics.json, frame.png (increment-1 `--demo` path).
    #[arg(long, default_value = "artifacts")]
    out: PathBuf,
    #[arg(long, default_value_t = DEFAULT_STEPS)]
    steps: u32,
    #[arg(long, default_value_t = FRAME_WIDTH)]
    width: u32,
    #[arg(long, default_value_t = FRAME_HEIGHT)]
    height: u32,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Increment 1 demo: write scene, step physics, render PNG.
    Demo {
        #[arg(long, default_value = "artifacts")]
        out: PathBuf,
        #[arg(long, default_value_t = DEFAULT_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 2: multi-body scene, step physics, render post-step PNG.
    Increment2 {
        #[arg(long, default_value = "artifacts/increment2")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT2_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Step a scene JSON and write a physics dump.
    Step {
        scene: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = DEFAULT_STEPS)]
        steps: u32,
    },
    /// Render a scene JSON (poses as given) to a PNG.
    Render {
        scene: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Optional physics dump whose poses replace scene body poses.
        #[arg(long)]
        physics: Option<PathBuf>,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 4: mesh body + primitives, step physics, render post-step PNG.
    Increment4 {
        #[arg(long, default_value = "artifacts/increment4")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT4_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 5: textured mesh + primitives, step physics, render post-step PNG.
    Increment5 {
        #[arg(long, default_value = "artifacts/increment5")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT5_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 6: two distinct meshes + primitives, step physics, render post-step PNG.
    Increment6 {
        #[arg(long, default_value = "artifacts/increment6")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT6_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 7: environment mesh as ground + props, step physics, render post-step PNG.
    Increment7 {
        #[arg(long, default_value = "artifacts/increment7")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT7_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 8: courtyard plus a glTF mesh body, step physics, render post-step PNG.
    Increment8 {
        #[arg(long, default_value = "artifacts/increment8")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT8_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 9: glTF pbrMetallicRoughness on the pillar, scene-JSON fallback.
    Increment9 {
        #[arg(long, default_value = "artifacts/increment9")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT9_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 10: courtyard plus a local point light (keep the directional).
    Increment10 {
        #[arg(long, default_value = "artifacts/increment10")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT10_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 11: courtyard point light casts shadows.
    Increment11 {
        #[arg(long, default_value = "artifacts/increment11")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT11_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 12: orbit 8 cameras around the increment-11 courtyard pose.
    Increment12 {
        #[arg(long, default_value = "artifacts/increment12")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT12_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 13: glTF metallicRoughnessTexture on the courtyard pillar.
    Increment13 {
        #[arg(long, default_value = "artifacts/increment13")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT13_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 14: glTF normalTexture (tangent-space bump) on the courtyard pillar.
    Increment14 {
        #[arg(long, default_value = "artifacts/increment14")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT14_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 15: courtyard ball shove, 8 physics frames, one camera.
    Increment15 {
        #[arg(long, default_value = "artifacts/increment15")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT15_FRAMES)]
        frames: u32,
        #[arg(long, default_value_t = INCREMENT15_STRIDE)]
        stride: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 16: courtyard still; glTF emissiveFactor * emissiveTexture on the pillar.
    Increment16 {
        #[arg(long, default_value = "artifacts/increment16")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT16_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 17: courtyard still plus a BLEND glass pane (continue ray / blend).
    Increment17 {
        #[arg(long, default_value = "artifacts/increment17")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT17_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 18: courtyard still; rectangular area light with a soft penumbra.
    Increment18 {
        #[arg(long, default_value = "artifacts/increment18")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT18_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 19: courtyard still; glTF occlusionTexture (AO) on the pillar.
    Increment19 {
        #[arg(long, default_value = "artifacts/increment19")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT19_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 20: courtyard still; pane transmission + Snell refraction (authored IOR).
    Increment20 {
        #[arg(long, default_value = "artifacts/increment20")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT20_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 21: courtyard still plus two authored mesh bodies (crate + bench).
    Increment21 {
        #[arg(long, default_value = "artifacts/increment21")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT21_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 22: courtyard still; gold ball has authored clearcoat + clearcoat_roughness.
    Increment22 {
        #[arg(long, default_value = "artifacts/increment22")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT22_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 23: courtyard still; bench has authored fabric/velvet sheen.
    Increment23 {
        #[arg(long, default_value = "artifacts/increment23")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT23_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 24: courtyard still; pane has KHR_materials_volume (Beer-Lambert).
    Increment24 {
        #[arg(long, default_value = "artifacts/increment24")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT24_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 25: courtyard still; gold ball has authored anisotropy (brushed metal).
    Increment25 {
        #[arg(long, default_value = "artifacts/increment25")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT25_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 26: courtyard still; gold ball has authored thin-film iridescence.
    Increment26 {
        #[arg(long, default_value = "artifacts/increment26")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT26_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 27: courtyard still; pane has authored KHR_materials_dispersion.
    Increment27 {
        #[arg(long, default_value = "artifacts/increment27")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT27_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 28: courtyard plus a hanging lantern on the pillar (authored hinge).
    Increment28 {
        #[arg(long, default_value = "artifacts/increment28")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT28_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 29: courtyard plus a drawer on the crate (authored slider).
    Increment29 {
        #[arg(long, default_value = "artifacts/increment29")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT29_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 30: courtyard lantern hinge has an authored motor.
    Increment30 {
        #[arg(long, default_value = "artifacts/increment30")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT30_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 31: courtyard plus a charm hanging from the lantern (authored ball socket).
    Increment31 {
        #[arg(long, default_value = "artifacts/increment31")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT31_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 32: courtyard plus a drawer-open sensor volume (authored trigger).
    Increment32 {
        #[arg(long, default_value = "artifacts/increment32")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT32_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 33: courtyard lantern self-glows and lights nearby surfaces (mesh light).
    Increment33 {
        #[arg(long, default_value = "artifacts/increment33")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT33_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 34: courtyard crate–drawer slider motor drives the drawer closed.
    Increment34 {
        #[arg(long, default_value = "artifacts/increment34")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT34_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 35: courtyard authors a drawer_probe raycast that hits the closed drawer.
    Increment35 {
        #[arg(long, default_value = "artifacts/increment35")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT35_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 36: courtyard authors a drawer_sweep shapecast that hits the closed drawer.
    Increment36 {
        #[arg(long, default_value = "artifacts/increment36")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT36_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 37: courtyard authors a crate lid welded with a fixed joint.
    Increment37 {
        #[arg(long, default_value = "artifacts/increment37")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT37_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 38: courtyard authors an impulse that rolls the gold ball off its seat.
    Increment38 {
        #[arg(long, default_value = "artifacts/increment38")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT38_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 39: courtyard authors a kinematic platform that slides +X.
    Increment39 {
        #[arg(long, default_value = "artifacts/increment39")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT39_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 40: courtyard authors a dynamic rider on the kinematic platform.
    Increment40 {
        #[arg(long, default_value = "artifacts/increment40")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT40_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 41: courtyard authors a limited hinge gate in the foreground.
    Increment41 {
        #[arg(long, default_value = "artifacts/increment41")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT41_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 42: courtyard authors a brass bob hung from the gate (rope).
    Increment42 {
        #[arg(long, default_value = "artifacts/increment42")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT42_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 43: courtyard snaps the gate–bob rope so the brass bob falls.
    Increment43 {
        #[arg(long, default_value = "artifacts/increment43")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT43_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 44: courtyard hangs a cork from the gate on a spring.
    Increment44 {
        #[arg(long, default_value = "artifacts/increment44")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT44_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 45: courtyard drives the gate hinge to a target angle.
    Increment45 {
        #[arg(long, default_value = "artifacts/increment45")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT45_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 46: courtyard re-aims the single camera at the gate cluster.
    Increment46 {
        #[arg(long, default_value = "artifacts/increment46")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT46_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 47: courtyard records started/stopped contact events in the dump.
    Increment47 {
        #[arg(long, default_value = "artifacts/increment47")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT47_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 48: courtyard walks a coral character controller on the floor.
    Increment48 {
        #[arg(long, default_value = "artifacts/increment48")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT48_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 49: courtyard walker ignores a yellow bar via collision groups.
    Increment49 {
        #[arg(long, default_value = "artifacts/increment49")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT49_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 50: spawn a gold token, despawn the yellow bar.
    Increment50 {
        #[arg(long, default_value = "artifacts/increment50")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT50_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 51: walker picks up the gold token on overlap.
    Increment51 {
        #[arg(long, default_value = "artifacts/increment51")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT51_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 52: camera follows the walker.
    Increment52 {
        #[arg(long, default_value = "artifacts/increment52")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT52_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 53: run until the token is picked.
    Increment53 {
        #[arg(long, default_value = "artifacts/increment53")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT53_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 54: a second scene, a short stone lane.
    Increment54 {
        #[arg(long, default_value = "artifacts/increment54")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT54_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 55: lane play-until entered (exit pad past the token).
    Increment55 {
        #[arg(long, default_value = "artifacts/increment55")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT55_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 56: walker carries the token to the exit.
    Increment56 {
        #[arg(long, default_value = "artifacts/increment56")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT56_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 57: walker drops the token at the exit.
    Increment57 {
        #[arg(long, default_value = "artifacts/increment57")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT57_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 58: amber npc walks -x and passes the coral walker.
    Increment58 {
        #[arg(long, default_value = "artifacts/increment58")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT58_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 59: named lane scene (catalog id lane).
    Increment59 {
        #[arg(long, default_value = "artifacts/increment59")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT59_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 60: run the catalog courtyard (sim --scene courtyard).
    Increment60 {
        #[arg(long, default_value = "artifacts/increment60")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT60_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 61: lane authors transition to courtyard.
    Increment61 {
        #[arg(long, default_value = "artifacts/increment61")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT61_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 62: carry held items across a transition.
    Increment62 {
        #[arg(long, default_value = "artifacts/increment62")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT62_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Increment 63: authorable win (delivered / token).
    Increment63 {
        #[arg(long, default_value = "artifacts/increment63")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT63_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// List named scenes in the catalog (courtyard, lane).
    Scenes {
        /// Write `[{id:courtyard},{id:lane}]`. If omitted, print ids to stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Increment 3: write ramp scene, simulate over time, render frame PNGs.
    Increment3 {
        #[arg(long, default_value = "artifacts/increment3")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT3_FRAMES)]
        frames: u32,
        #[arg(long, default_value_t = INCREMENT3_STRIDE)]
        stride: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
    },
    /// Simulate a scene over time: N frame PNGs + trajectory dump.
    /// With `--scene <id>`, look up a catalog scene and step+render it
    /// (dump.scene stamped with the catalog key when the authored scene
    /// has no id).
    Sim {
        /// Scene JSON file (file-based trajectory sim). Omit when --scene is set.
        #[arg(required_unless_present = "catalog_scene")]
        scene: Option<PathBuf>,
        /// Catalog scene id (courtyard, lane).
        #[arg(long = "scene", value_name = "ID", id = "catalog_scene")]
        catalog_scene: Option<String>,
        #[arg(long, default_value_t = INCREMENT3_FRAMES)]
        frames: u32,
        #[arg(long, default_value = "artifacts/increment60")]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT3_STRIDE)]
        stride: u32,
        #[arg(long, default_value_t = INCREMENT60_STEPS)]
        steps: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
        /// Prior physics dump whose dump.held bodies are injected into the destination.
        #[arg(long)]
        carry: Option<PathBuf>,
    },
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Some(Command::Demo {
            out,
            steps,
            width,
            height,
        }) => run_increment1(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment2 {
            out,
            steps,
            width,
            height,
        }) => run_increment2(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Step { scene, out, steps }) => {
            step_scene_file(&scene, &out, steps, DEFAULT_DT).map(|()| {
                println!("wrote {}", out.display());
                None
            })
        }
        Some(Command::Render {
            scene,
            out,
            physics,
            width,
            height,
        }) => render_scene_file(&scene, &out, physics.as_deref(), width, height).map(|()| {
            println!("wrote {}", out.display());
            None
        }),
        Some(Command::Increment4 {
            out,
            steps,
            width,
            height,
        }) => run_increment4(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment5 {
            out,
            steps,
            width,
            height,
        }) => run_increment5(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment6 {
            out,
            steps,
            width,
            height,
        }) => run_increment6(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment7 {
            out,
            steps,
            width,
            height,
        }) => run_increment7(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment8 {
            out,
            steps,
            width,
            height,
        }) => run_increment8(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment9 {
            out,
            steps,
            width,
            height,
        }) => run_increment9(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment10 {
            out,
            steps,
            width,
            height,
        }) => run_increment10(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment11 {
            out,
            steps,
            width,
            height,
        }) => run_increment11(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment12 {
            out,
            steps,
            width,
            height,
        }) => run_increment12(&out, steps, DEFAULT_DT, width, height).map(|paths| {
            println!("wrote {}", paths.scene.display());
            println!("wrote {}", paths.physics.display());
            for frame in &paths.frames {
                println!("wrote {}", frame.display());
            }
            None
        }),
        Some(Command::Increment13 {
            out,
            steps,
            width,
            height,
        }) => run_increment13(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment14 {
            out,
            steps,
            width,
            height,
        }) => run_increment14(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment15 {
            out,
            frames,
            stride,
            width,
            height,
        }) => run_increment15(&out, frames, stride, DEFAULT_DT, width, height).map(|paths| {
            println!("wrote {}", paths.scene.display());
            println!("wrote {}", paths.physics.display());
            for frame in &paths.frames {
                println!("wrote {}", frame.display());
            }
            None
        }),
        Some(Command::Increment16 {
            out,
            steps,
            width,
            height,
        }) => run_increment16(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment17 {
            out,
            steps,
            width,
            height,
        }) => run_increment17(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment18 {
            out,
            steps,
            width,
            height,
        }) => run_increment18(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment19 {
            out,
            steps,
            width,
            height,
        }) => run_increment19(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment20 {
            out,
            steps,
            width,
            height,
        }) => run_increment20(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment21 {
            out,
            steps,
            width,
            height,
        }) => run_increment21(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment22 {
            out,
            steps,
            width,
            height,
        }) => run_increment22(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment23 {
            out,
            steps,
            width,
            height,
        }) => run_increment23(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment24 {
            out,
            steps,
            width,
            height,
        }) => run_increment24(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment25 {
            out,
            steps,
            width,
            height,
        }) => run_increment25(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment26 {
            out,
            steps,
            width,
            height,
        }) => run_increment26(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment27 {
            out,
            steps,
            width,
            height,
        }) => run_increment27(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment28 {
            out,
            steps,
            width,
            height,
        }) => run_increment28(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment29 {
            out,
            steps,
            width,
            height,
        }) => run_increment29(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment30 {
            out,
            steps,
            width,
            height,
        }) => run_increment30(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment31 {
            out,
            steps,
            width,
            height,
        }) => run_increment31(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment32 {
            out,
            steps,
            width,
            height,
        }) => run_increment32(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment33 {
            out,
            steps,
            width,
            height,
        }) => run_increment33(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment34 {
            out,
            steps,
            width,
            height,
        }) => run_increment34(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment35 {
            out,
            steps,
            width,
            height,
        }) => run_increment35(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment36 {
            out,
            steps,
            width,
            height,
        }) => run_increment36(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment37 {
            out,
            steps,
            width,
            height,
        }) => run_increment37(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment38 {
            out,
            steps,
            width,
            height,
        }) => run_increment38(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment39 {
            out,
            steps,
            width,
            height,
        }) => run_increment39(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment40 {
            out,
            steps,
            width,
            height,
        }) => run_increment40(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment41 {
            out,
            steps,
            width,
            height,
        }) => run_increment41(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment42 {
            out,
            steps,
            width,
            height,
        }) => run_increment42(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment43 {
            out,
            steps,
            width,
            height,
        }) => run_increment43(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment44 {
            out,
            steps,
            width,
            height,
        }) => run_increment44(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment45 {
            out,
            steps,
            width,
            height,
        }) => run_increment45(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment46 {
            out,
            steps,
            width,
            height,
        }) => run_increment46(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment47 {
            out,
            steps,
            width,
            height,
        }) => run_increment47(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment48 {
            out,
            steps,
            width,
            height,
        }) => run_increment48(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment49 {
            out,
            steps,
            width,
            height,
        }) => run_increment49(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment50 {
            out,
            steps,
            width,
            height,
        }) => run_increment50(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment51 {
            out,
            steps,
            width,
            height,
        }) => run_increment51(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment52 {
            out,
            steps,
            width,
            height,
        }) => run_increment52(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment53 {
            out,
            steps,
            width,
            height,
        }) => run_increment53(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment54 {
            out,
            steps,
            width,
            height,
        }) => run_increment54(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment55 {
            out,
            steps,
            width,
            height,
        }) => run_increment55(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment56 {
            out,
            steps,
            width,
            height,
        }) => run_increment56(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment57 {
            out,
            steps,
            width,
            height,
        }) => run_increment57(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment58 {
            out,
            steps,
            width,
            height,
        }) => run_increment58(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment59 {
            out,
            steps,
            width,
            height,
        }) => run_increment59(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment60 {
            out,
            steps,
            width,
            height,
        }) => run_increment60(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment61 {
            out,
            steps,
            width,
            height,
        }) => run_increment61(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment62 {
            out,
            steps,
            width,
            height,
        }) => run_increment62(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Increment63 {
            out,
            steps,
            width,
            height,
        }) => run_increment63(&out, steps, DEFAULT_DT, width, height).map(Some),
        Some(Command::Scenes { out }) => {
            if let Some(path) = out {
                write_scenes_catalog(&path).map(|()| {
                    println!("wrote {}", path.display());
                    None
                })
            } else {
                for id in catalog_ids() {
                    println!("{id}");
                }
                Ok(None)
            }
        }
        Some(Command::Increment3 {
            out,
            frames,
            stride,
            width,
            height,
        }) => run_increment3(&out, frames, stride, DEFAULT_DT, width, height).map(|paths| {
            println!("wrote {}", paths.scene.display());
            println!("wrote {}", paths.trajectory.display());
            println!("wrote {}", paths.frame.display());
            println!("wrote {}", paths.frames_dir.display());
            None
        }),
        Some(Command::Sim {
            scene,
            catalog_scene,
            frames,
            out,
            stride,
            steps,
            width,
            height,
            carry,
        }) => {
            let carry_res: Result<Option<PhysicsDump>, String> = match carry {
                Some(path) => std::fs::read_to_string(&path)
                    .map_err(|e| format!("read carry {path:?}: {e}"))
                    .and_then(|txt| {
                        serde_json::from_str(&txt).map_err(|e| format!("parse carry: {e}"))
                    })
                    .map(Some),
                None => Ok(None),
            };
            match carry_res {
                Err(e) => Err(e),
                Ok(carry_dump) => if let Some(id) = catalog_scene {
                match run_catalog_scene_with_carry(&id, &out, steps, DEFAULT_DT, width, height, carry_dump.as_ref()) {
                    Ok(paths) => {
                        println!("wrote {}", paths.scene.display());
                        println!("wrote {}", paths.physics.display());
                        println!("wrote {}", paths.frame.display());
                        Ok(None)
                    }
                    Err(e) => {
                        if e.starts_with("unknown scene id:") {
                            eprintln!("error: {e}");
                            std::process::exit(1);
                        }
                        Err(e)
                    }
                }
            } else {
                let scene = scene.expect("scene file required without --scene");
                sim_scene_file(&scene, &out, frames, stride, DEFAULT_DT, width, height).map(|paths| {
                    println!("wrote {}", paths.trajectory.display());
                    println!("wrote {}", paths.frame.display());
                    println!("wrote {}", paths.frames_dir.display());
                    None
                })
            }
            }
        }
        None if args.demo => {
            run_increment1(&args.out, args.steps, DEFAULT_DT, args.width, args.height).map(Some)
        }
        None => {
            eprintln!("usage: agent-rig <demo|increment2|increment3|increment4|increment5|increment6|increment7|increment8|increment9|increment10|increment11|increment12|increment13|increment14|increment15|increment16|increment17|increment18|increment19|increment20|increment21|increment22|increment23|increment24|increment25|increment26|increment27|increment28|increment29|increment30|increment31|increment32|increment33|increment34|increment35|increment36|increment37|increment38|increment39|increment40|increment41|increment42|increment43|increment44|increment45|increment46|increment47|increment48|increment49|increment50|increment51|increment52|increment53|increment54|increment55|increment56|increment57|increment58|increment59|increment60|increment61|increment62|increment63|scenes|sim|step|render> …  (or --demo for increment 1)");
            std::process::exit(2);
        }
    };

    match result {
        Ok(Some(paths)) => {
            println!("wrote {}", paths.scene.display());
            println!("wrote {}", paths.physics.display());
            println!("wrote {}", paths.frame.display());
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
