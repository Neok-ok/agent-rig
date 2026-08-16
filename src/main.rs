use std::path::PathBuf;

use agent_rig::{
    render_scene_file, run_increment1, run_increment2, step_scene_file, DEFAULT_DT, DEFAULT_STEPS,
    FRAME_HEIGHT, FRAME_WIDTH, INCREMENT2_STEPS,
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
        None if args.demo => {
            run_increment1(&args.out, args.steps, DEFAULT_DT, args.width, args.height).map(Some)
        }
        None => {
            eprintln!("usage: agent-rig <demo|increment2|step|render> …  (or --demo for increment 1)");
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
