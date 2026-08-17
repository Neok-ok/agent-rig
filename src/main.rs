use std::path::PathBuf;

use agent_rig::{
    render_scene_file, run_increment1, run_increment2, run_increment3, run_increment4,
    run_increment5, run_increment6, run_increment7, run_increment8, run_increment9, run_increment10, run_increment11, run_increment12, run_increment13, run_increment14, run_increment15, run_increment16, sim_scene_file, step_scene_file, DEFAULT_DT,
    DEFAULT_STEPS, FRAME_HEIGHT, FRAME_WIDTH, INCREMENT2_STEPS, INCREMENT3_FRAMES, INCREMENT3_STRIDE,
    INCREMENT4_STEPS, INCREMENT5_STEPS, INCREMENT6_STEPS, INCREMENT7_STEPS, INCREMENT8_STEPS,
    INCREMENT9_STEPS, INCREMENT10_STEPS, INCREMENT11_STEPS, INCREMENT12_STEPS, INCREMENT13_STEPS,
    INCREMENT14_STEPS, INCREMENT15_FRAMES, INCREMENT15_STRIDE, INCREMENT16_STEPS,
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
            eprintln!("usage: agent-rig <demo|increment2|increment3|increment4|increment5|increment6|increment7|increment8|increment9|increment10|increment11|increment12|increment13|increment14|increment15|increment16|sim|step|render> …  (or --demo for increment 1)");
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
