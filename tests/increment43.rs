use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment19_scene_json, increment20_scene_json,
    increment21_scene_json, increment22_scene_json, increment23_scene_json,
    increment24_scene_json, increment25_scene_json, increment26_scene_json,
    increment27_scene_json, increment28_scene_json, increment29_scene_json,
    increment30_scene_json, increment31_scene_json, increment32_scene_json,
    increment33_scene_json, increment34_scene_json, increment35_scene_json,
    increment36_scene_json, increment37_scene_json, increment38_scene_json,
    increment39_scene_json, increment40_scene_json, increment41_scene_json,
    increment42_scene, increment42_scene_json, increment43_scene,
    increment43_scene_json, parse_scene, run_increment43, step_physics,
    Impulse, Joint, Light, Shape, DEFAULT_DT, INCREMENT42_STEPS, INCREMENT43_STEPS,
};

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn assert_ball_impulse(imp: &Impulse) {
    assert_eq!(imp.body, "ball");
    let lin = imp.linear;
    assert!(
        (lin[0] - 1.8).abs() < 1e-5 && (lin[1] - 0.4).abs() < 1e-5 && (lin[2] - 0.5).abs() < 1e-5,
        "ball impulse linear should be [1.8, 0.4, 0.5], got {lin:?}"
    );
}

fn assert_bob_impulse(imp: &Impulse) {
    assert_eq!(imp.body, "bob");
    let lin = imp.linear;
    assert!(
        (lin[0] - 0.0).abs() < 1e-5 && (lin[1] + 4.0).abs() < 1e-5 && (lin[2] - 1.6).abs() < 1e-5,
        "bob impulse linear should be [0.0, -4.0, 1.6], got {lin:?}"
    );
}

fn assert_increment42_impulses(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 1, "increment42 must keep exactly the ball impulse, got {}", impulses.len());
    assert_ball_impulse(&impulses[0]);
}

fn assert_increment43_impulses(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 2, "increment43 must keep ball + bob impulses, got {}", impulses.len());
    assert_ball_impulse(&impulses[0]);
    assert_bob_impulse(&impulses[1]);
}

fn assert_platform(body: &agent_rig::Body) {
    assert_eq!(body.id, "platform");
    assert!(body.kinematic, "platform must be kinematic");
    let vel = body.linear_velocity;
    assert!(
        (vel[0] - 0.45).abs() < 1e-5 && vel[1].abs() < 1e-5 && vel[2].abs() < 1e-5,
        "platform linear_velocity should be [0.45, 0, 0], got {vel:?}"
    );
}

fn assert_rider(body: &agent_rig::Body) {
    assert_eq!(body.id, "rider");
    assert!(!body.kinematic, "rider must not be kinematic");
    assert!((body.mass - 0.35).abs() < 1e-5, "rider mass should be 0.35, got {}", body.mass);
}

fn assert_gate(body: &agent_rig::Body) {
    assert_eq!(body.id, "gate");
    assert!(!body.kinematic, "gate must not be kinematic");
    match body.shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.06).abs() < 1e-5
                    && (size[1] - 0.72).abs() < 1e-5
                    && (size[2] - 0.42).abs() < 1e-5,
                "gate box size should be [0.06, 0.72, 0.42], got {size:?}"
            );
        }
        _ => panic!("gate should be a box, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] - 0.35).abs() < 1e-5 && (p[1] - 0.40).abs() < 1e-5 && (p[2] - 1.75).abs() < 1e-5,
        "gate position should be [0.35, 0.40, 1.75], got {p:?}"
    );
}

fn assert_bob(body: &agent_rig::Body) {
    assert_eq!(body.id, "bob");
    assert!(!body.kinematic, "bob must not be kinematic, got kinematic={}", body.kinematic);
    match body.shape {
        Shape::Sphere { radius } => {
            assert!((radius - 0.08).abs() < 1e-5, "bob sphere radius should be 0.08, got {radius}");
        }
        _ => panic!("bob should be a sphere, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] - 0.35).abs() < 1e-5 && (p[1] - 0.88).abs() < 1e-5 && (p[2] - 1.75).abs() < 1e-5,
        "bob position should be [0.35, 0.88, 1.75], got {p:?}"
    );
    assert!((body.mass - 0.2).abs() < 1e-5, "bob mass should be 0.2, got {}", body.mass);
}

fn gate_hinge(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 3], [f32; 2], f32, f32) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a, body_b, anchor, axis, limits, motor_target_velocity, motor_max_force,
        } = j
        {
            if body_a == "ground" && body_b == "gate" {
                let lim = limits.expect("ground-gate hinge must author limits");
                return (*anchor, *axis, lim, *motor_target_velocity, *motor_max_force);
            }
        }
    }
    panic!("scene missing ground-gate hinge");
}

fn distance_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], f32, f32) {
    for j in &scene.joints {
        if let Joint::Distance { body_a, body_b, anchor, rest_length, break_force } = j {
            return (body_a, body_b, *anchor, *rest_length, *break_force);
        }
    }
    panic!("scene missing distance joint");
}

fn slider_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], [f32; 2], f32, f32) {
    for j in &scene.joints {
        if let Joint::Slider { body_a, body_b, axis, limits, motor_target_velocity, motor_max_force, .. } = j {
            return (body_a, body_b, *axis, *limits, *motor_target_velocity, *motor_max_force);
        }
    }
    panic!("scene missing slider joint");
}

fn fixed_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3]) {
    for j in &scene.joints {
        if let Joint::Fixed { body_a, body_b, anchor } = j {
            return (body_a, body_b, *anchor);
        }
    }
    panic!("scene missing fixed joint");
}

#[test]
fn increment43_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 25] = [
        ("18", increment18_scene_json()),
        ("19", increment19_scene_json()),
        ("20", increment20_scene_json()),
        ("21", increment21_scene_json()),
        ("22", increment22_scene_json()),
        ("23", increment23_scene_json()),
        ("24", increment24_scene_json()),
        ("25", increment25_scene_json()),
        ("26", increment26_scene_json()),
        ("27", increment27_scene_json()),
        ("28", increment28_scene_json()),
        ("29", increment29_scene_json()),
        ("30", increment30_scene_json()),
        ("31", increment31_scene_json()),
        ("32", increment32_scene_json()),
        ("33", increment33_scene_json()),
        ("34", increment34_scene_json()),
        ("35", increment35_scene_json()),
        ("36", increment36_scene_json()),
        ("37", increment37_scene_json()),
        ("38", increment38_scene_json()),
        ("39", increment39_scene_json()),
        ("40", increment40_scene_json()),
        ("41", increment41_scene_json()),
        ("42", increment42_scene_json()),
    ];
    for (name, json) in prior_jsons {
        assert!(!json.contains("\"break_force\""), "must not mutate increment {name} scene JSON with break_force");
        assert!(!json.contains("\"broken_joints\""), "must not mutate increment {name} scene JSON with broken_joints");
    }

    let live42 = increment42_scene();
    let live43 = increment43_scene();
    assert_eq!(live42.lights.len(), live43.lights.len(), "increment43 must not add lights[] entries vs increment 42");
    assert_eq!(live42.bodies.len(), live43.bodies.len(), "increment43 must not add a body vs increment 42");
    assert_eq!(live42.joints.len(), live43.joints.len(), "increment43 must not add a joint vs increment 42");
    assert_eq!(live42.impulses.len() + 1, live43.impulses.len(), "increment43 must append only the bob impulse vs increment 42");
    assert_eq!(live42.camera.position, live43.camera.position, "camera must stay increment-42");
    assert_eq!(live42.camera.look_at, live43.camera.look_at);
}

#[test]
fn increment43_authors_breakable_rope_and_bob_impulse() {
    let parsed = parse_scene(increment43_scene_json()).expect("increment43 JSON should parse");
    assert_bob(body_by_id(&parsed, "bob"));
    assert_gate(body_by_id(&parsed, "gate"));
    assert_rider(body_by_id(&parsed, "rider"));
    assert_platform(body_by_id(&parsed, "platform"));
    let (a, b, anchor, rest, brk) = distance_of(&parsed);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5 && (anchor[1] - 0.76).abs() < 1e-5 && (anchor[2] - 1.75).abs() < 1e-5,
        "distance anchor should be [0.35, 0.76, 1.75], got {anchor:?}"
    );
    assert!((rest - 0.38).abs() < 1e-5, "distance rest_length should be 0.38, got {rest}");
    assert!((brk - 1.5).abs() < 0.35, "increment43 Distance break_force should be ~1.5, got {brk}");
    assert_increment43_impulses(&parsed.impulses);
    assert_eq!(parsed.camera.position, increment42_scene().camera.position);
    assert_eq!(parsed.camera.look_at, increment42_scene().camera.look_at);

    let live = increment43_scene();
    assert_bob(body_by_id(&live, "bob"));
    assert_gate(body_by_id(&live, "gate"));
    assert_rider(body_by_id(&live, "rider"));
    assert_platform(body_by_id(&live, "platform"));
    let (a, b, anchor, rest, brk) = distance_of(&live);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5 && (anchor[1] - 0.76).abs() < 1e-5 && (anchor[2] - 1.75).abs() < 1e-5,
        "live distance anchor {anchor:?}"
    );
    assert!((rest - 0.38).abs() < 1e-5, "live rest_length {rest}");
    assert!((brk - 1.5).abs() < 0.35, "live increment43 Distance break_force should be ~1.5, got {brk}");
    assert_eq!(live.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(live.camera.look_at, [0.1, 0.38, 0.0]);
    assert_increment43_impulses(&live.impulses);

    let prior = increment42_scene();
    let (_a, _b, _anchor, _rest, brk42) = distance_of(&prior);
    assert!(brk42.abs() < 1e-5, "increment42_scene Distance must stay unbreakable (break_force 0), got {brk42}");
    assert_increment42_impulses(&prior.impulses);
    let ser42 = serde_json::to_string(&prior).expect("serialize increment42_scene");
    assert!(!ser42.contains("\"break_force\""), "increment42_scene() must serialize without break_force");
    assert!(!ser42.contains("\"broken_joints\""), "increment42_scene() must serialize without broken_joints");
}

#[test]
fn increment43_keeps_courtyard() {
    let scene = increment43_scene();
    assert!(scene.bodies.len() >= 15, "scene must have courtyard + gate + bob, got {}", scene.bodies.len());
    let inc42 = increment42_scene();
    assert_eq!(scene.lights.len(), inc42.lights.len(), "no extra lights[] entries vs increment 42");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert_eq!(scene.bodies.len(), inc42.bodies.len(), "no new body vs increment 42");
    assert_eq!(scene.joints.len(), inc42.joints.len(), "no new joint vs increment 42");
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.triggers.len(), 1);
    assert_eq!(scene.triggers[0].id, "drawer_open");
    assert_increment43_impulses(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_bob(body_by_id(&scene, "bob"));

    let (_anchor, _axis, limits, gv, gf) = gate_hinge(&scene);
    assert!((limits[0] - 0.0).abs() < 1e-5 && (limits[1] - 1.15).abs() < 1e-5);
    assert!((gv - 1.4).abs() < 1e-5 && (gf - 5.0).abs() < 1e-5);
    let (sa, sb, _axis, slims, sv, sf) = slider_of(&scene);
    assert_eq!(sa, "crate");
    assert_eq!(sb, "drawer");
    assert!(slims[1] > slims[0]);
    assert!((sv - (-2.0)).abs() < 1e-5 && (sf - 6.0).abs() < 1e-5);
    let (fa, fb, _fanchor) = fixed_of(&scene);
    assert_eq!(fa, "crate");
    assert_eq!(fb, "lid");
    assert!(scene.bodies.iter().any(|b| b.id == "lid"));
    assert!(scene.bodies.iter().any(|b| b.id == "charm"));
    assert!(scene.bodies.iter().any(|b| b.id == "lantern"));
    assert!(scene.bodies.iter().any(|b| b.id == "drawer"));
    assert!(scene.joints.iter().any(|j| matches!(j, Joint::Ball { .. })));
    assert_eq!(scene.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(scene.camera.look_at, [0.1, 0.38, 0.0]);
    let has_dir = scene.lights.iter().any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");
}

#[test]
fn increment42_rope_stays_unbreakable_and_bob_hangs() {
    let scene = increment42_scene();
    let (_a, _b, _anchor, _rest, brk) = distance_of(&scene);
    assert!(brk.abs() < 1e-5, "increment42_scene break_force must be 0, got {brk}");
    assert_increment42_impulses(&scene.impulses);
    let dump = step_physics(&scene, INCREMENT42_STEPS, DEFAULT_DT).expect("physics");
    assert!(dump.broken_joints.is_empty(), "increment42 dump must not record broken_joints, got {:?}", dump.broken_joints);
    let bob = dump.bodies.iter().find(|b| b.id == "bob").expect("dump missing bob");
    assert!(bob.position[1] > 0.22, "increment42 bob should stay hanging well above 0.22, got {:?}", bob.position);
    assert!((bob.position[1] - 0.38).abs() < 0.12, "increment42 bob should stay near hang height y≈0.38, got {:?}", bob.position);
    let dist_j = dump.joints.iter().find(|j| j.kind == "distance" && j.body_a == "gate" && j.body_b == "bob")
        .expect("increment42 dump must keep the live gate-bob distance joint");
    let dump_rest = dist_j.rest_length.expect("dump distance joint must record rest_length");
    assert!((dump_rest - 0.38).abs() < 1e-5);
}

#[test]
fn increment43_rope_snaps_and_bob_falls() {
    let scene = increment43_scene();
    let dump = step_physics(&scene, INCREMENT43_STEPS, DEFAULT_DT).expect("physics");
    assert_increment43_impulses(&dump.impulses);

    let bob = dump.bodies.iter().find(|b| b.id == "bob").expect("dump missing bob");
    assert!(!bob.kinematic, "dump bob must not be kinematic");
    assert!(bob.position[1] < 0.22, "bob should fall onto the bowl (y < 0.22), got {:?}", bob.position);

    assert!(
        dump.joints.iter().all(|j| !(j.kind == "distance" && j.body_a == "gate" && j.body_b == "bob")),
        "dump joints must omit the broken gate-bob distance, got {:?}",
        dump.joints
    );
    assert!(
        dump.broken_joints.iter().any(|j| j.kind == "distance" && j.body_a == "gate" && j.body_b == "bob"),
        "broken_joints must include gate-bob distance, got {:?}",
        dump.broken_joints
    );

    let authored_x = body_by_id(&scene, "platform").position[0];
    let platform = dump.bodies.iter().find(|b| b.id == "platform").expect("dump missing platform");
    assert!(platform.kinematic, "dump platform must record kinematic: true");
    assert!((platform.position[0] - authored_x).abs() > 0.4, "platform should slide +X by more than 0.4, authored_x={authored_x} got {:?}", platform.position);

    let authored_rider_x = body_by_id(&scene, "rider").position[0];
    let rider = dump.bodies.iter().find(|b| b.id == "rider").expect("dump missing rider");
    assert!(!rider.kinematic, "dump rider must not be kinematic");
    assert!((rider.position[0] - authored_rider_x).abs() > 0.35, "rider should ride +X by more than 0.35, got {:?}", rider.position);
    assert!(rider.position[1] > 0.10, "rider COM should stay on the slab, got {:?}", rider.position);

    let gate_j = dump.joints.iter().find(|j| j.kind == "hinge" && j.body_a == "ground" && j.body_b == "gate")
        .expect("dump missing ground-gate hinge");
    let glim = gate_j.limits.expect("dump gate hinge must record limits");
    assert!((glim[0] - 0.0).abs() < 1e-5 && (glim[1] - 1.15).abs() < 1e-5);
    let angle = gate_j.angle.expect("dump gate hinge must record angle");
    assert!((angle - 1.15).abs() <= 0.2, "gate hinge angle should be within 0.2 of 1.15, got {angle}");
    assert!(angle <= 1.30, "gate hinge angle must not pass 1.30, got {angle}");

    let ball = dump.bodies.iter().find(|b| b.id == "ball").expect("dump missing ball");
    assert!((ball.position[0] + 1.1).abs() > 0.25, "gold ball COM should roll off the seat at x=-1.1, got {:?}", ball.position);

    let lid = dump.bodies.iter().find(|b| b.id == "lid").expect("dump missing lid");
    assert!(lid.position[1] > 0.20, "lid COM should stay on the crate, got {:?}", lid.position);

    let drawer = dump.bodies.iter().find(|b| b.id == "drawer").expect("dump missing drawer");
    assert!(drawer.position[2] < 1.15, "closed drawer COM should sit near z=1.02, got {:?}", drawer.position);

    let max_toi = scene.shapecasts[0].max_toi;
    let hit = dump.sweep_hits.iter().find(|h| h.sweep == "drawer_sweep").expect("dump sweep_hits must record drawer_sweep");
    assert_eq!(hit.body, "drawer");
    assert!(hit.toi > 0.0 && hit.toi < max_toi);
    let ray_hit = dump.ray_hits.iter().find(|h| h.ray == "drawer_probe").expect("dump ray_hits must still record drawer_probe");
    assert_eq!(ray_hit.body, "drawer");
}

#[test]
fn increment43_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment43.sh");
    assert!(script.is_file(), "scripts/increment43.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment43-threejs.sh");
    assert!(three.is_file(), "scripts/increment43-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment43-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment43(&out, INCREMENT43_STEPS, DEFAULT_DT, 200, 112).expect("run_increment43");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_bob(body_by_id(&scene, "bob"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_platform(body_by_id(&scene, "platform"));
    let (a, b, _anchor, rest, brk) = distance_of(&scene);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!((rest - 0.38).abs() < 1e-5);
    assert!((brk - 1.5).abs() < 0.35, "written scene break_force ~1.5, got {brk}");
    assert_increment43_impulses(&scene.impulses);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.lights.len(), 2);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 15);
    let bob_state = v["bodies"].as_array().unwrap().iter().find(|b| b["id"] == "bob").expect("dump should record the bob");
    assert_eq!(bob_state["kinematic"].as_bool().unwrap_or(false), false);
    let by = bob_state["position"][1].as_f64().unwrap();
    assert!(by < 0.22, "written dump bob y should be on the bowl (< 0.22), got {by}");
    let joints = v["joints"].as_array().expect("dump must have joints array");
    assert!(
        joints.iter().all(|j| !(j["kind"] == "distance" && j["body_a"] == "gate" && j["body_b"] == "bob")),
        "written dump joints must omit the live gate-bob distance"
    );
    let broken = v["broken_joints"].as_array().expect("dump must record broken_joints");
    assert!(
        broken.iter().any(|j| j["kind"] == "distance" && j["body_a"] == "gate" && j["body_b"] == "bob"),
        "written dump broken_joints must include gate-bob distance, got {broken:?}"
    );
    let gate_j = joints.iter().find(|j| j["kind"] == "hinge" && j["body_a"] == "ground" && j["body_b"] == "gate")
        .expect("dump joints must record the ground-gate hinge");
    let gangle = gate_j["angle"].as_f64().expect("written dump gate hinge must record angle");
    assert!((gangle - 1.15).abs() <= 0.2);
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
