use std::path::PathBuf;

use agent_rig::{run_increment1, DEFAULT_DT, DEFAULT_STEPS, FRAME_HEIGHT, FRAME_WIDTH};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "agent-rig", about = "Scene JSON + physics dump + headless PNG")]
struct Args {
    /// Use the built-in demo scene (ball falling onto a ground box).
    #[arg(long)]
    demo: bool,
    /// Output directory for scene.json, physics.json, frame.png.
    #[arg(long, default_value = "artifacts")]
    out: PathBuf,
    #[arg(long, default_value_t = DEFAULT_STEPS)]
    steps: u32,
    #[arg(long, default_value_t = FRAME_WIDTH)]
    width: u32,
    #[arg(long, default_value_t = FRAME_HEIGHT)]
    height: u32,
}

fn main() {
    let args = Args::parse();
    if !args.demo {
        eprintln!("increment 1: pass --demo (only the demo scene is wired)");
        std::process::exit(2);
    }
    match run_increment1(&args.out, args.steps, DEFAULT_DT, args.width, args.height) {
        Ok(paths) => {
            println!("wrote {}", paths.scene.display());
            println!("wrote {}", paths.physics.display());
            println!("wrote {}", paths.frame.display());
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
