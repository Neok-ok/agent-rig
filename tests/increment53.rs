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
    increment42_scene_json, increment43_scene_json, increment44_scene_json,
    increment45_scene_json, increment46_scene_json, increment47_scene_json,
    increment48_scene_json, increment49_scene_json, increment50_scene_json,
    increment51_scene_json, increment52_scene, increment52_scene_json, increment53_scene, increment53_scene_json,
    parse_scene, run_increment53, step_physics, Impulse, Joint, Light, Shape,
    DEFAULT_DT, INCREMENT52_STEPS, INCREMENT53_STEPS,
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

fn assert_impulses(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 2, "must keep ball + bob impulses, got {}", impulses.len());
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
}

fn assert_bob(body: &agent_rig::Body) {
    assert_eq!(body.id, "bob");
    match body.shape {
        Shape::Sphere { radius } => {
            assert!((radius - 0.08).abs() < 1e-5, "bob sphere radius should be 0.08, got {radius}");
        }
        _ => panic!("bob should be a sphere, got {:?}", body.shape),
    }
}

fn assert_cork(body: &agent_rig::Body) {
    assert_eq!(body.id, "cork");
    match body.shape {
        Shape::Sphere { radius } => {
            assert!((radius - 0.14).abs() < 1e-5, "cork sphere radius should be 0.14, got {radius}");
        }
        _ => panic!("cork should be a sphere, got {:?}", body.shape),
    }
}

fn assert_walker(body: &agent_rig::Body) {
    assert_eq!(body.id, "walker");
    assert!(!body.kinematic, "walker must not use increment-39 kinematic linvel drive");
    assert!(body.mass.abs() < 1e-5, "walker mass should be 0, got {}", body.mass);
    match body.shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.18).abs() < 1e-5
                    && (size[1] - 0.36).abs() < 1e-5
                    && (size[2] - 0.18).abs() < 1e-5,
                "walker box size should be [0.18, 0.36, 0.18], got {size:?}"
            );
        }
        _ => panic!("walker should be a box, got {:?}", body.shape),
    }
    let ctrl = body
        .controller
        .as_ref()
        .expect("walker must author a controller");
    let v = ctrl.desired_velocity;
    assert!(
        (v[0] + 0.55).abs() < 0.06 && v[1].abs() < 1e-5 && v[2].abs() < 1e-5,
        "walker desired_velocity should be about [-0.55, 0, 0], got {v:?}"
    );
    let pos = body.position;
    assert!(
        (pos[0] - 1.15).abs() < 0.08 && (pos[1] - 0.20).abs() < 0.08 && (pos[2] - 1.45).abs() < 0.08,
        "walker start should be about [1.15, 0.20, 1.45], got {pos:?}"
    );
    let alb = body.material.albedo;
    assert!(
        alb[0] > 0.7 && alb[1] < 0.4 && alb[2] > 0.3,
        "walker albedo should be warm coral/magenta, got {alb:?}"
    );
}

fn assert_walker_groups(body: &agent_rig::Body) {
    assert_eq!(body.collision_groups.membership, 2, "walker membership should be WALKER=2");
    assert_eq!(body.collision_groups.filter, 1, "walker filter should be GROUND=1");
}

fn assert_bar(body: &agent_rig::Body) {
    assert_eq!(body.id, "bar");
    assert!(!body.kinematic, "bar must be a static (mass 0) box, not increment-39 kinematic");
    assert!(body.mass.abs() < 1e-5, "bar mass should be 0, got {}", body.mass);
    assert!(body.controller.is_none(), "bar must not author a controller");
    match body.shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.08).abs() < 1e-5
                    && (size[1] - 0.40).abs() < 1e-5
                    && (size[2] - 0.28).abs() < 1e-5,
                "bar box size should be [0.08, 0.40, 0.28], got {size:?}"
            );
        }
        _ => panic!("bar should be a box, got {:?}", body.shape),
    }
    let pos = body.position;
    assert!(
        (pos[0] - 0.55).abs() < 0.08 && (pos[1] - 0.22).abs() < 0.08 && (pos[2] - 1.45).abs() < 0.08,
        "bar pose should be about [0.55, 0.22, 1.45], got {pos:?}"
    );
    let alb = body.material.albedo;
    assert!(
        alb[0] > 0.8 && alb[1] > 0.65 && alb[2] < 0.35,
        "bar albedo should be yellow, got {alb:?}"
    );
    assert_eq!(body.collision_groups.membership, 4, "bar membership should be PROP=4");
    assert_eq!(body.collision_groups.filter, 0xFFFF, "bar filter should be 0xFFFF");
}

fn pair_involves(a: &str, b: &str, x: &str, y: &str) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

fn gate_hinge(scene: &agent_rig::Scene) -> ([f32; 2], f32, f32, Option<f32>) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            limits,
            motor_target_velocity,
            motor_max_force,
            motor_target_position,
            ..
        } = j
        {
            if body_a == "ground" && body_b == "gate" {
                let lim = limits.expect("ground-gate hinge must author limits");
                return (lim, *motor_target_velocity, *motor_max_force, *motor_target_position);
            }
        }
    }
    panic!("scene missing ground-gate hinge");
}

fn spring_of(scene: &agent_rig::Scene) -> (&str, &str, f32, f32, f32) {
    for j in &scene.joints {
        if let Joint::Spring {
            body_a, body_b, rest_length, stiffness, damping, ..
        } = j
        {
            return (body_a, body_b, *rest_length, *stiffness, *damping);
        }
    }
    panic!("scene missing spring joint");
}

fn distance_of(scene: &agent_rig::Scene) -> (&str, &str, f32, f32) {
    for j in &scene.joints {
        if let Joint::Distance { body_a, body_b, rest_length, break_force, .. } = j {
            return (body_a, body_b, *rest_length, *break_force);
        }
    }
    panic!("scene missing distance joint");
}

fn slider_of(scene: &agent_rig::Scene) -> (&str, &str, f32, f32) {
    for j in &scene.joints {
        if let Joint::Slider { body_a, body_b, motor_target_velocity, motor_max_force, .. } = j {
            return (body_a, body_b, *motor_target_velocity, *motor_max_force);
        }
    }
    panic!("scene missing slider joint");
}

fn fixed_of(scene: &agent_rig::Scene) -> (&str, &str) {
    for j in &scene.joints {
        if let Joint::Fixed { body_a, body_b, .. } = j {
            return (body_a, body_b);
        }
    }
    panic!("scene missing fixed joint");
}

fn assert_no_extra_camera_keys(json: &str, label: &str) {
    assert!(
        !json.contains("\"cameras\""),
        "{label} must not author a cameras[] array"
    );
    let camera_keys = json.matches("\"camera\"").count();
    assert_eq!(
        camera_keys, 1,
        "{label} must have exactly one camera key, got {camera_keys}"
    );
}


fn assert_token(body: &agent_rig::Body) {
    assert_eq!(body.id, "token");
    assert!(body.mass.abs() < 1e-5, "token mass should be 0, got {}", body.mass);
    match body.shape {
        Shape::Sphere { radius } => {
            assert!((radius - 0.10).abs() < 1e-5, "token radius should be 0.10, got {radius}");
        }
        _ => panic!("token should be a sphere, got {:?}", body.shape),
    }
    let pos = body.position;
    assert!(
        (pos[0] - 0.70).abs() < 0.08 && (pos[1] - 0.12).abs() < 0.08 && (pos[2] - 1.45).abs() < 0.08,
        "token pose should be about [0.70, 0.12, 1.45], got {pos:?}"
    );
    let alb = body.material.albedo;
    assert!(
        alb[0] > 0.85 && alb[1] > 0.65 && alb[2] < 0.40,
        "token albedo should be gold, got {alb:?}"
    );
    assert_eq!(body.collision_groups.membership, 4, "token membership should be PROP=4");
    assert_eq!(body.collision_groups.filter, 0xFFFF, "token filter should be 0xFFFF");
}


#[test]
fn increment53_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 35] = [
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
        ("43", increment43_scene_json()),
        ("44", increment44_scene_json()),
        ("45", increment45_scene_json()),
        ("46", increment46_scene_json()),
        ("47", increment47_scene_json()),
        ("48", increment48_scene_json()),
        ("49", increment49_scene_json()),
        ("50", increment50_scene_json()),
        ("51", increment51_scene_json()),
        ("52", increment52_scene_json()),
    ];
    for (name, json) in prior_jsons {
        assert!(
            !json.contains("\"play_until\""),
            "must not mutate increment {name} scene JSON with play_until"
        );
    }
    let live52 = increment52_scene();
    let live53 = increment53_scene();
    assert!(live52.play_until.is_none(), "increment52_scene must stay play-until-free");
    assert!(live52.camera.follow.is_some(), "increment52 HAS follow");
    assert!(live53.play_until.is_some(), "increment53_scene must author play_until");
    assert!(live53.camera.follow.is_some(), "increment53 keeps increment-52 follow");
    assert!(live52.record_contact_events);
    assert!(live53.record_contact_events);
    assert_eq!(live52.lights.len(), live53.lights.len());
    assert_eq!(live52.bodies.len(), live53.bodies.len());
    assert_eq!(live52.joints.len(), live53.joints.len());
    assert_eq!(live52.pickups.len(), live53.pickups.len());
    assert_eq!(live52.triggers.len(), live53.triggers.len());
    assert_eq!(live52.camera.position, live53.camera.position);
    assert_eq!(live52.camera.look_at, live53.camera.look_at);
}

#[test]
fn increment53_scene_adds_play_until() {
    let parsed = parse_scene(increment53_scene_json()).expect("increment53 JSON should parse");
    assert_eq!(parsed.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(parsed.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert_no_extra_camera_keys(increment53_scene_json(), "increment53_scene_json");
    assert!(increment53_scene_json().contains("\"play_until\""));
    assert!(increment53_scene_json().contains("\"follow\""));
    let until = parsed.play_until.as_ref().expect("increment53 must author play_until");
    assert_eq!(until.kind, "picked_up");
    assert_eq!(until.body, "token");
    let follow = parsed.camera.follow.as_ref().expect("increment53 camera keeps follow");
    assert_eq!(follow.body, "walker");
    assert!((follow.offset[0] - 1.20).abs() < 1e-5);
    assert!((follow.offset[1] - 0.90).abs() < 1e-5);
    assert!((follow.offset[2] - 1.50).abs() < 1e-5);
    assert!(parsed.triggers.iter().any(|t| t.id == "token_zone"));
    assert_eq!(parsed.pickups[0].body, "token");
    assert_eq!(parsed.spawns[0].at_step, 30);
    assert_eq!(parsed.despawns[0].body, "bar");
    assert_walker(body_by_id(&parsed, "walker"));

    let live = increment53_scene();
    let u = live.play_until.as_ref().expect("live play_until");
    assert_eq!(u.kind, "picked_up");
    assert_eq!(u.body, "token");
    let f = live.camera.follow.as_ref().expect("live follow");
    assert_eq!(f.body, "walker");
    assert_eq!(f.offset, [1.20, 0.90, 1.50]);
    let prior = increment52_scene();
    assert!(prior.play_until.is_none(), "increment52 HAS follow — play_until stays none");
    assert!(prior.camera.follow.is_some(), "increment52 HAS follow");
    assert!(!increment52_scene_json().contains("\"play_until\""));
    assert!(increment52_scene_json().contains("\"follow\""));
}

#[test]
fn increment53_keeps_courtyard() {
    let scene = increment53_scene();
    let inc52 = increment52_scene();
    assert_eq!(scene.lights.len(), 2);
    assert_eq!(scene.bodies.len(), inc52.bodies.len());
    assert_eq!(scene.joints.len(), inc52.joints.len());
    assert_eq!(scene.pickups.len(), inc52.pickups.len());
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert!(scene.triggers.iter().any(|t| t.id == "drawer_open"));
    assert!(scene.triggers.iter().any(|t| t.id == "token_zone"));
    assert_impulses(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_cork(body_by_id(&scene, "cork"));
    assert_walker(body_by_id(&scene, "walker"));
    assert_bar(body_by_id(&scene, "bar"));
    assert_eq!(scene.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(scene.camera.look_at, [0.35, 0.42, 1.55]);
    assert!(scene.camera.follow.is_some());
    assert!(scene.play_until.is_some());
}

#[test]
fn increment52_dump_is_fixed_step() {
    let scene = increment52_scene();
    assert!(scene.play_until.is_none(), "increment52 HAS follow — assert play_until.is_none not follow.is_none");
    assert!(scene.camera.follow.is_some(), "increment52 HAS follow");
    let dump = step_physics(&scene, INCREMENT52_STEPS, DEFAULT_DT).expect("increment52 physics");
    assert_eq!(dump.steps, 120);
    assert!(dump.stopped.is_none());
    let json = serde_json::to_value(&dump).expect("serialize increment52 dump");
    assert!(json.get("stopped").is_none(), "increment52 dump JSON must omit stopped");
    assert!(dump.camera.is_some(), "increment52 dump has follow-cam");
    assert!(dump.bodies.iter().all(|b| b.id != "token"));
    assert!(dump.picked_up.iter().any(|p| p.id == "token" && p.by == "walker"));
    assert!(dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30));
    assert!(dump.despawned.iter().any(|s| s.id == "bar" && s.at_step == 80));
}

#[test]
fn increment53_physics_play_until() {
    let scene = increment53_scene();
    let authored_x = body_by_id(&scene, "walker").position[0];
    let dump = step_physics(&scene, INCREMENT53_STEPS, DEFAULT_DT).expect("physics");
    assert!(
        (30..=31).contains(&dump.steps),
        "play_until should stop at pickup, dump.steps 30..=31, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("increment53 dump must record stopped");
    assert_eq!(stopped.kind, "picked_up");
    assert_eq!(stopped.body, "token");
    assert!((30..=31).contains(&stopped.at_step), "stopped.at_step 30..=31, got {}", stopped.at_step);

    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump missing walker");
    assert!(walker.position[0] > 0.6 && walker.position[0] < authored_x, "walker should still be left of start, got {}", walker.position[0]);
    assert!(walker.position[1] >= 0.14 && walker.position[1] <= 0.28, "walker.y on floor, got {}", walker.position[1]);
    let ctrl = dump.controllers.iter().find(|c| c.id == "walker").expect("walker controller");
    assert!(ctrl.grounded);

    let cam = dump.camera.as_ref().expect("dump.camera must be set when follow is authored");
    let expect_pos = [
        walker.position[0] + 1.20,
        walker.position[1] + 0.90,
        walker.position[2] + 1.50,
    ];
    let expect_look = [
        walker.position[0],
        walker.position[1] + 0.15,
        walker.position[2],
    ];
    for i in 0..3 {
        assert!((cam.position[i] - expect_pos[i]).abs() < 0.08, "dump.camera.position[{i}] got {} want {}", cam.position[i], expect_pos[i]);
        assert!((cam.look_at[i] - expect_look[i]).abs() < 0.08, "dump.camera.look_at[{i}] got {} want {}", cam.look_at[i], expect_look[i]);
    }

    assert!(dump.bodies.iter().all(|b| b.id != "token"), "pickup leftover: no token");
    assert!(dump.bodies.iter().any(|b| b.id == "bar"), "bar must still be present");
    assert!(dump.picked_up.iter().any(|p| p.id == "token" && p.by == "walker" && (30..=31).contains(&p.at_step)));
    assert!(dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30));
    assert!(dump.despawned.is_empty(), "bar despawn is at 80; play_until stops first");
    assert!(!dump.contact_events.is_empty());
}

#[test]
fn increment53_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment53.sh");
    assert!(script.is_file(), "scripts/increment53.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment53-threejs.sh");
    assert!(three.is_file(), "scripts/increment53-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment53-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment53(&out, INCREMENT53_STEPS, DEFAULT_DT, 200, 112).expect("run_increment53");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert_no_extra_camera_keys(&scene_txt, "written scene.json");
    assert!(scene_txt.contains("\"follow\""));
    assert!(scene_txt.contains("\"play_until\""));
    let scene = parse_scene(&scene_txt).expect("written scene parses");
    let until = scene.play_until.as_ref().expect("written play_until");
    assert_eq!(until.kind, "picked_up");
    assert_eq!(until.body, "token");
    let follow = scene.camera.follow.as_ref().expect("written follow");
    assert_eq!(follow.body, "walker");
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    let steps = v["steps"].as_u64().unwrap();
    assert!((30..=31).contains(&steps), "written dump.steps 30..=31, got {steps}");
    let stopped = v.get("stopped").expect("written dump must have stopped");
    assert_eq!(stopped["kind"], "picked_up");
    assert_eq!(stopped["body"], "token");
    let cam = v.get("camera").expect("written dump must have camera");
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().all(|b| b["id"] != "token"));
    assert!(bodies.iter().any(|b| b["id"] == "bar"), "bar must still be present");
    let walker = bodies.iter().find(|b| b["id"] == "walker").unwrap();
    let wx = walker["position"][0].as_f64().unwrap();
    let wy = walker["position"][1].as_f64().unwrap();
    let wz = walker["position"][2].as_f64().unwrap();
    let px = cam["position"][0].as_f64().unwrap();
    let py = cam["position"][1].as_f64().unwrap();
    let pz = cam["position"][2].as_f64().unwrap();
    assert!((px - (wx + 1.20)).abs() < 0.08 && (py - (wy + 0.90)).abs() < 0.08 && (pz - (wz + 1.50)).abs() < 0.08);
    let lx = cam["look_at"][0].as_f64().unwrap();
    let ly = cam["look_at"][1].as_f64().unwrap();
    let lz = cam["look_at"][2].as_f64().unwrap();
    assert!((lx - wx).abs() < 0.08 && (ly - (wy + 0.15)).abs() < 0.08 && (lz - wz).abs() < 0.08);
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
