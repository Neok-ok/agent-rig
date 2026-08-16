use std::path::PathBuf;

use agent_rig::{
    render_scene_file, run_increment1, run_increment2, run_increment3, run_increment4,
    run_increment5, run_increment6, run_increment7, run_increment8, run_increment9, sim_scene_file, step_scene_file, DEFAULT_DT,
    DEFAULT_STEPS, FRAME_HEIGHT, FRAME_WIDTH, INCREMENT2_STEPS, INCREMENT3_FRAMES, INCREMENT3_STRIDE,
    INCREMENT4_STEPS, INCREMENT5_STEPS, INCREMENT6_STEPS, INCREMENT7_STEPS, INCREMENT8_STEPS,
    INCREMENT9_STEPS,
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
    Sim {
        scene: PathBuf,
        #[arg(long, default_value_t = INCREMENT3_FRAMES)]
        frames: u32,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = INCREMENT3_STRIDE)]
        stride: u32,
        #[arg(long, default_value_t = FRAME_WIDTH)]
        width: u32,
        #[arg(long, default_value_t = FRAME_HEIGHT)]
        height: u32,
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
            frames,
            out,
            stride,
            width,
            height,
        }) => sim_scene_file(&scene, &out, frames, stride, DEFAULT_DT, width, height).map(|paths| {
            println!("wrote {}", paths.trajectory.display());
            println!("wrote {}", paths.frame.display());
            println!("wrote {}", paths.frames_dir.display());
            None
        }),
        None if args.demo => {
            run_increment1(&args.out, args.steps, DEFAULT_DT, args.width, args.height).map(Some)
        }
        None => {
            eprintln!("usage: agent-rig <demo|increment2|increment3|increment4|increment5|increment6|increment7|increment8|increment9|sim|step|render> …  (or --demo for increment 1)");
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
