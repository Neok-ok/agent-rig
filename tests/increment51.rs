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
    increment48_scene_json, increment49_scene_json, increment50_scene,
    increment50_scene_json, increment51_scene, increment51_scene_json,
    parse_scene, run_increment51, step_physics, Impulse, Joint, Light, Shape,
    DEFAULT_DT, INCREMENT50_STEPS, INCREMENT51_STEPS,
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
fn increment51_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 33] = [
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
    ];
    for (name, json) in prior_jsons {
        assert!(
            !json.contains("\"pickups\""),
            "must not mutate increment {name} scene JSON with pickups"
        );
        assert!(
            !json.contains("\"token_zone\""),
            "must not mutate increment {name} scene JSON with token_zone"
        );
    }
    let live50 = increment50_scene();
    let live51 = increment51_scene();
    assert!(live50.pickups.is_empty(), "increment50_scene must stay pickup-free");
    assert!(live50.triggers.iter().all(|t| t.id != "token_zone"));
    assert!(live50.record_contact_events);
    assert!(live51.record_contact_events);
    assert_eq!(live50.lights.len(), live51.lights.len());
    assert_eq!(live50.bodies.len(), live51.bodies.len());
    assert_eq!(live50.joints.len(), live51.joints.len());
    assert_eq!(live50.spawns.len(), live51.spawns.len());
    assert_eq!(live50.despawns.len(), live51.despawns.len());
    assert_eq!(live50.camera.position, live51.camera.position);
    assert_eq!(live50.camera.look_at, live51.camera.look_at);
    assert_eq!(live50.triggers.len() + 1, live51.triggers.len());
}

#[test]
fn increment51_scene_adds_token_zone_and_pickups() {
    let parsed = parse_scene(increment51_scene_json()).expect("increment51 JSON should parse");
    assert_eq!(parsed.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(parsed.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert_no_extra_camera_keys(increment51_scene_json(), "increment51_scene_json");
    assert!(increment51_scene_json().contains("\"pickups\""));
    assert!(increment51_scene_json().contains("\"token_zone\""));
    assert_eq!(parsed.pickups.len(), 1);
    assert_eq!(parsed.pickups[0].body, "token");
    assert_eq!(parsed.pickups[0].trigger, "token_zone");
    assert_eq!(parsed.pickups[0].by, "walker");
    let zone = parsed.triggers.iter().find(|t| t.id == "token_zone").expect("missing token_zone");
    match zone.shape {
        Shape::Box { size } => {
            assert!((size[0] - 0.40).abs() < 1e-5 && (size[1] - 0.40).abs() < 1e-5 && (size[2] - 0.40).abs() < 1e-5);
        }
        _ => panic!("token_zone should be a box"),
    }
    assert!((zone.position[0] - 0.70).abs() < 1e-5 && (zone.position[1] - 0.12).abs() < 1e-5 && (zone.position[2] - 1.45).abs() < 1e-5);
    assert!(parsed.triggers.iter().any(|t| t.id == "drawer_open"));
    assert_eq!(parsed.spawns[0].at_step, 30);
    assert_token(&parsed.spawns[0].body);
    assert_eq!(parsed.despawns[0].at_step, 80);
    assert_eq!(parsed.despawns[0].body, "bar");
    assert_walker(body_by_id(&parsed, "walker"));
    assert_bar(body_by_id(&parsed, "bar"));
    assert_impulses(&parsed.impulses);

    let live = increment51_scene();
    assert_eq!(live.pickups.len(), 1);
    assert_eq!(live.pickups[0].body, "token");
    assert!(live.triggers.iter().any(|t| t.id == "token_zone"));
    assert!(live.triggers.iter().any(|t| t.id == "drawer_open"));
    let prior = increment50_scene();
    assert!(prior.pickups.is_empty());
    assert!(prior.triggers.iter().all(|t| t.id != "token_zone"));
}

#[test]
fn increment51_keeps_courtyard() {
    let scene = increment51_scene();
    let inc50 = increment50_scene();
    assert_eq!(scene.lights.len(), 2);
    assert_eq!(scene.bodies.len(), inc50.bodies.len());
    assert_eq!(scene.joints.len(), inc50.joints.len());
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert!(scene.triggers.iter().any(|t| t.id == "drawer_open"));
    assert_impulses(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_cork(body_by_id(&scene, "cork"));
    assert_walker(body_by_id(&scene, "walker"));
    assert_bar(body_by_id(&scene, "bar"));
    let (sa, sb, rest, stiff, damp) = spring_of(&scene);
    assert_eq!((sa, sb), ("gate", "cork"));
    assert!((rest - 0.42).abs() < 1e-5 && (stiff - 40.0).abs() < 1e-5 && (damp - 4.0).abs() < 1e-5);
    assert_eq!(scene.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(scene.camera.look_at, [0.35, 0.42, 1.55]);
}

#[test]
fn increment50_dump_has_token_no_picked_up() {
    let scene = increment50_scene();
    assert!(scene.pickups.is_empty());
    assert!(scene.triggers.iter().all(|t| t.id != "token_zone"));
    let dump = step_physics(&scene, INCREMENT50_STEPS, DEFAULT_DT).expect("increment50 physics");
    assert!(dump.bodies.iter().any(|b| b.id == "token"), "increment50 dump must still include token");
    assert!(dump.bodies.iter().all(|b| b.id != "bar"));
    assert!(dump.picked_up.is_empty());
    let json = serde_json::to_value(&dump).expect("serialize increment50 dump");
    assert!(json.get("picked_up").is_none(), "increment50 dump JSON must omit picked_up");
    assert!(dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30));
    assert!(dump.despawned.iter().any(|s| s.id == "bar" && s.at_step == 80));
}

#[test]
fn increment51_physics_picks_up_token() {
    let scene = increment51_scene();
    let authored_x = body_by_id(&scene, "walker").position[0];
    let dump = step_physics(&scene, INCREMENT51_STEPS, DEFAULT_DT).expect("physics");
    assert!(dump.bodies.iter().all(|b| b.id != "token"), "dump.bodies must omit picked-up token");
    assert!(dump.bodies.iter().all(|b| b.id != "bar"), "bar still despawned");
    assert!(
        dump.picked_up.iter().any(|p| p.id == "token" && p.by == "walker" && p.at_step >= 30 && p.at_step <= 80),
        "dump.picked_up must include token by walker at_step 30-80, got {:?}",
        dump.picked_up
    );
    assert!(dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30));
    assert_eq!(dump.despawned.len(), 1);
    assert!(dump.despawned.iter().any(|s| s.id == "bar" && s.at_step == 80));
    assert!(dump.despawned.iter().all(|s| s.id != "token"), "token must not also be in despawned");

    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump missing walker");
    assert!(walker.position[0] <= authored_x - 0.4, "walker.x should move >= 0.4 toward -X, got {}", walker.position[0]);
    assert!(walker.position[1] >= 0.14 && walker.position[1] <= 0.28, "walker.y on floor, got {}", walker.position[1]);
    let ctrl = dump.controllers.iter().find(|c| c.id == "walker").expect("walker controller");
    assert!(ctrl.grounded);
    assert!(!dump.contact_events.is_empty());
    let bob = dump.bodies.iter().find(|b| b.id == "bob").expect("bob");
    assert!(bob.position[1] < 0.22);
    assert!(dump.broken_joints.iter().any(|j| j.kind == "distance" && j.body_a == "gate" && j.body_b == "bob"));
}

#[test]
fn increment51_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment51.sh");
    assert!(script.is_file(), "scripts/increment51.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment51-threejs.sh");
    assert!(three.is_file(), "scripts/increment51-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment51-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment51(&out, INCREMENT51_STEPS, DEFAULT_DT, 200, 112).expect("run_increment51");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert_no_extra_camera_keys(&scene_txt, "written scene.json");
    assert!(scene_txt.contains("\"pickups\"") && scene_txt.contains("\"token_zone\""));
    let scene = parse_scene(&scene_txt).expect("written scene parses");
    assert_eq!(scene.pickups[0].body, "token");
    assert!(scene.triggers.iter().any(|t| t.id == "token_zone"));
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().all(|b| b["id"] != "token"));
    assert!(bodies.iter().all(|b| b["id"] != "bar"));
    let picked = v["picked_up"].as_array().expect("written dump must have picked_up");
    assert!(picked.iter().any(|p| p["id"] == "token" && p["by"] == "walker"));
    let at = picked.iter().find(|p| p["id"] == "token").unwrap()["at_step"].as_u64().unwrap();
    assert!((30..=80).contains(&at), "pickup at_step should be 30-80, got {at}");
    let walker = bodies.iter().find(|b| b["id"] == "walker").unwrap();
    let wx = walker["position"][0].as_f64().unwrap();
    let wy = walker["position"][1].as_f64().unwrap();
    assert!(wx <= 1.15 - 0.4);
    assert!(wy >= 0.14 && wy <= 0.28);
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
}
