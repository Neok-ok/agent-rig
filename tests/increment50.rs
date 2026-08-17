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
    increment48_scene_json, increment49_scene, increment49_scene_json,
    increment50_scene, increment50_scene_json, parse_scene, run_increment50,
    step_physics, Impulse, Joint, Light, Shape, DEFAULT_DT, INCREMENT49_STEPS,
    INCREMENT50_STEPS,
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
fn increment50_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 32] = [
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
    ];
    for (name, json) in prior_jsons {
        assert!(
            !json.contains("\"spawns\""),
            "must not mutate increment {name} scene JSON with spawns"
        );
        assert!(
            !json.contains("\"despawns\""),
            "must not mutate increment {name} scene JSON with despawns"
        );
        assert!(
            !json.contains("\"token\""),
            "must not mutate increment {name} scene JSON with token"
        );
    }

    let live49 = increment49_scene();
    let live50 = increment50_scene();
    assert!(live49.spawns.is_empty(), "increment49_scene must stay spawn-free");
    assert!(live49.despawns.is_empty(), "increment49_scene must stay despawn-free");
    assert!(live49.bodies.iter().any(|b| b.id == "bar"), "increment49_scene must keep bar");
    assert!(live49.bodies.iter().all(|b| b.id != "token"), "increment49_scene must stay token-free");
    assert!(live49.record_contact_events);
    assert!(live50.record_contact_events, "increment50 must keep record_contact_events");
    assert_eq!(live49.lights.len(), live50.lights.len(), "increment50 must not add lights vs increment 49");
    assert_eq!(live49.bodies.len(), live50.bodies.len(), "increment50 must not add initial bodies vs increment 49");
    assert_eq!(live49.joints.len(), live50.joints.len(), "increment50 must not add joints vs increment 49");
    assert_eq!(live49.impulses.len(), live50.impulses.len());
    assert_eq!(live49.camera.position, live50.camera.position);
    assert_eq!(live49.camera.look_at, live50.camera.look_at);
    assert_eq!(live49.camera.fov_y_deg, live50.camera.fov_y_deg);
    let w49 = body_by_id(&live49, "walker");
    let w50 = body_by_id(&live50, "walker");
    assert_eq!(w49.position, w50.position, "increment50 must not change walker start");
    assert_eq!(
        w49.controller.as_ref().unwrap().desired_velocity,
        w50.controller.as_ref().unwrap().desired_velocity,
        "increment50 must not change walker controller"
    );
    assert_eq!(w49.collision_groups, w50.collision_groups);
}

#[test]
fn increment50_scene_adds_spawns_and_despawns() {
    let parsed = parse_scene(increment50_scene_json()).expect("increment50 JSON should parse");
    assert_eq!(parsed.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(parsed.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert_no_extra_camera_keys(increment50_scene_json(), "increment50_scene_json");
    assert!(increment50_scene_json().contains("\"spawns\""));
    assert!(increment50_scene_json().contains("\"despawns\""));
    assert!(increment50_scene_json().contains("\"token\""));
    assert_eq!(parsed.spawns.len(), 1);
    assert_eq!(parsed.spawns[0].at_step, 30);
    assert_token(&parsed.spawns[0].body);
    assert_eq!(parsed.despawns.len(), 1);
    assert_eq!(parsed.despawns[0].at_step, 80);
    assert_eq!(parsed.despawns[0].body, "bar");
    assert_walker(body_by_id(&parsed, "walker"));
    assert_walker_groups(body_by_id(&parsed, "walker"));
    assert_bar(body_by_id(&parsed, "bar"));
    let (limits, vel, force, pos) = gate_hinge(&parsed);
    assert!((limits[0] - 0.0).abs() < 1e-5 && (limits[1] - 1.15).abs() < 1e-5);
    assert!(vel.abs() < 1e-5);
    assert!((force - 5.0).abs() < 1e-5);
    let target = pos.expect("increment50 JSON gate must keep motor_target_position");
    assert!((target - 0.55).abs() < 1e-5);
    assert_cork(body_by_id(&parsed, "cork"));
    assert_bob(body_by_id(&parsed, "bob"));
    assert_gate(body_by_id(&parsed, "gate"));
    assert_rider(body_by_id(&parsed, "rider"));
    assert_platform(body_by_id(&parsed, "platform"));
    assert_impulses(&parsed.impulses);

    let live = increment50_scene();
    assert_eq!(live.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(live.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((live.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert!(live.record_contact_events);
    assert_eq!(live.spawns.len(), 1);
    assert_eq!(live.spawns[0].at_step, 30);
    assert_token(&live.spawns[0].body);
    assert_eq!(live.despawns.len(), 1);
    assert_eq!(live.despawns[0].at_step, 80);
    assert_eq!(live.despawns[0].body, "bar");
    assert_walker(body_by_id(&live, "walker"));
    assert_bar(body_by_id(&live, "bar"));
    assert_impulses(&live.impulses);

    let prior = increment49_scene();
    assert!(prior.spawns.is_empty());
    assert!(prior.despawns.is_empty());
    assert!(prior.bodies.iter().any(|b| b.id == "bar"));
    assert!(prior.bodies.iter().all(|b| b.id != "token"));
}

#[test]
fn increment50_keeps_courtyard() {
    let scene = increment50_scene();
    let inc49 = increment49_scene();
    assert_eq!(scene.lights.len(), inc49.lights.len(), "no extra lights[] entries vs increment 49");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert_eq!(scene.bodies.len(), inc49.bodies.len(), "no extra initial bodies vs increment 49");
    assert_eq!(scene.joints.len(), inc49.joints.len(), "no new joints vs increment 49");
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.triggers.len(), 1);
    assert_eq!(scene.triggers[0].id, "drawer_open");
    assert_impulses(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_cork(body_by_id(&scene, "cork"));
    assert_walker(body_by_id(&scene, "walker"));
    assert_walker_groups(body_by_id(&scene, "walker"));
    assert_bar(body_by_id(&scene, "bar"));

    let (sa, sb, rest, stiff, damp) = spring_of(&scene);
    assert_eq!(sa, "gate");
    assert_eq!(sb, "cork");
    assert!((rest - 0.42).abs() < 1e-5);
    assert!((stiff - 40.0).abs() < 1e-5);
    assert!((damp - 4.0).abs() < 1e-5);
    let (da, db, drest, dbrk) = distance_of(&scene);
    assert_eq!(da, "gate");
    assert_eq!(db, "bob");
    assert!((drest - 0.38).abs() < 1e-5);
    assert!((dbrk - 1.5).abs() < 0.35);
    let (sla, slb, sv, sf) = slider_of(&scene);
    assert_eq!(sla, "crate");
    assert_eq!(slb, "drawer");
    assert!((sv - (-2.0)).abs() < 1e-5 && (sf - 6.0).abs() < 1e-5);
    let (fa, fb) = fixed_of(&scene);
    assert_eq!(fa, "crate");
    assert_eq!(fb, "lid");
    assert!(scene.bodies.iter().any(|b| b.id == "lid"));
    assert!(scene.bodies.iter().any(|b| b.id == "charm"));
    assert!(scene.bodies.iter().any(|b| b.id == "lantern"));
    assert!(scene.bodies.iter().any(|b| b.id == "drawer"));
    assert!(scene.joints.iter().any(|j| matches!(j, Joint::Ball { .. })));
    assert_eq!(scene.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(scene.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((scene.camera.fov_y_deg - 40.0).abs() < 1e-5);
    let has_dir = scene.lights.iter().any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");
}

#[test]
fn increment49_dump_has_bar_no_timed_events() {
    let scene = increment49_scene();
    assert!(scene.spawns.is_empty());
    assert!(scene.despawns.is_empty());
    assert!(scene.bodies.iter().any(|b| b.id == "bar"));
    assert!(scene.bodies.iter().all(|b| b.id != "token"));
    let dump = step_physics(&scene, INCREMENT49_STEPS, DEFAULT_DT).expect("increment49 physics");
    assert!(
        dump.bodies.iter().any(|b| b.id == "bar"),
        "increment49 dump must still include bar"
    );
    assert!(
        dump.bodies.iter().all(|b| b.id != "token"),
        "increment49 dump must stay without token"
    );
    assert!(dump.spawned.is_empty(), "increment49 dump must stay without spawned");
    assert!(dump.despawned.is_empty(), "increment49 dump must stay without despawned");
    let json = serde_json::to_value(&dump).expect("serialize increment49 dump");
    assert!(
        json.get("spawned").is_none(),
        "increment49 dump JSON must omit spawned when empty"
    );
    assert!(
        json.get("despawned").is_none(),
        "increment49 dump JSON must omit despawned when empty"
    );
    let scene_json = serde_json::to_string(&scene).expect("serialize increment49 scene");
    assert!(
        !scene_json.contains("\"spawns\""),
        "increment49 scene JSON must omit spawns when empty"
    );
    assert!(
        !scene_json.contains("\"despawns\""),
        "increment49 scene JSON must omit despawns when empty"
    );
    assert!(
        !scene_json.contains("\"token\""),
        "increment49 scene JSON must not contain token"
    );
}

#[test]
fn increment50_physics_spawns_token_despawns_bar() {
    let scene = increment50_scene();
    let authored_x = body_by_id(&scene, "walker").position[0];
    let dump = step_physics(&scene, INCREMENT50_STEPS, DEFAULT_DT).expect("physics");
    assert_impulses(&dump.impulses);

    assert!(
        dump.bodies.iter().any(|b| b.id == "token"),
        "dump.bodies must include token after spawn"
    );
    assert!(
        dump.bodies.iter().all(|b| b.id != "bar"),
        "dump.bodies must omit bar after despawn, got {:?}",
        dump.bodies.iter().map(|b| &b.id).collect::<Vec<_>>()
    );
    let token = dump.bodies.iter().find(|b| b.id == "token").unwrap();
    assert!(
        (token.position[0] - 0.70).abs() < 0.08
            && (token.position[1] - 0.12).abs() < 0.08
            && (token.position[2] - 1.45).abs() < 0.08,
        "token should stay near [0.70, 0.12, 1.45], got {:?}",
        token.position
    );
    assert_eq!(token.collision_groups.membership, 4);
    assert_eq!(token.collision_groups.filter, 0xFFFF);
    assert!(
        dump.spawned.iter().any(|s| s.id == "token" && s.at_step == 30),
        "dump.spawned must include token@30, got {:?}",
        dump.spawned
    );
    assert!(
        dump.despawned.iter().any(|s| s.id == "bar" && s.at_step == 80),
        "dump.despawned must include bar@80, got {:?}",
        dump.despawned
    );

    let walker = dump.bodies.iter().find(|b| b.id == "walker").expect("dump missing walker");
    assert!(
        walker.position[0] <= authored_x - 0.4,
        "walker.x should move at least 0.4 toward -X (authored {authored_x}, got {})",
        walker.position[0]
    );
    assert!(
        walker.position[1] >= 0.14 && walker.position[1] <= 0.28,
        "walker.y should stay on the floor [0.14, 0.28], got {}",
        walker.position[1]
    );
    let ctrl = dump
        .controllers
        .iter()
        .find(|c| c.id == "walker")
        .expect("dump.controllers must include walker");
    assert!(ctrl.grounded, "walker should be grounded after 120 steps");

    assert!(
        !dump.contact_events.is_empty(),
        "increment50 dump must still record contact_events"
    );
    assert!(
        dump.contact_events.iter().any(|e| {
            e.kind == "started" && pair_involves(&e.body_a, &e.body_b, "bob", "ground")
        }),
        "contact_events must include started involving bob and ground"
    );
    assert!(
        dump.contact_events.iter().any(|e| {
            e.kind == "started" && pair_involves(&e.body_a, &e.body_b, "rider", "platform")
        }),
        "contact_events must include started involving rider and platform"
    );

    let gate_j = dump
        .joints
        .iter()
        .find(|j| j.kind == "hinge" && j.body_a == "ground" && j.body_b == "gate")
        .expect("dump missing ground-gate hinge");
    let angle = gate_j.angle.expect("dump gate hinge must record angle");
    assert!((angle - 0.55).abs() <= 0.15, "gate hinge angle should be within 0.15 of 0.55, got {angle}");
    let cork = dump.bodies.iter().find(|b| b.id == "cork").expect("dump missing cork");
    assert!(cork.position[1] < 1.15 - 0.15, "cork should drop, got {:?}", cork.position);
    let bob = dump.bodies.iter().find(|b| b.id == "bob").expect("dump missing bob");
    assert!(bob.position[1] < 0.22, "bob should fall onto the bowl, got {:?}", bob.position);
    assert!(
        dump.broken_joints.iter().any(|j| j.kind == "distance" && j.body_a == "gate" && j.body_b == "bob"),
        "broken_joints must include gate-bob distance"
    );
    let rider = dump.bodies.iter().find(|b| b.id == "rider").expect("dump missing rider");
    let platform = dump.bodies.iter().find(|b| b.id == "platform").expect("dump missing platform");
    assert!(platform.kinematic);
    assert!(rider.position[0] > -0.55 + 0.2, "rider should still ride +X, got {:?}", rider.position);
    assert!(dump.ray_hits.iter().any(|h| h.ray == "drawer_probe"));
    assert!(dump.sweep_hits.iter().any(|h| h.sweep == "drawer_sweep"));
}

#[test]
fn increment50_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment50.sh");
    assert!(script.is_file(), "scripts/increment50.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment50-threejs.sh");
    assert!(three.is_file(), "scripts/increment50-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment50-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment50(&out, INCREMENT50_STEPS, DEFAULT_DT, 200, 112).expect("run_increment50");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");
    assert!(
        !out.join("frame_01.png").exists() && !out.join("cameras.json").exists(),
        "must not write a second frame or cameras file"
    );

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert_no_extra_camera_keys(&scene_txt, "written scene.json");
    assert!(scene_txt.contains("\"spawns\""), "written scene must author spawns");
    assert!(scene_txt.contains("\"despawns\""), "written scene must author despawns");
    assert!(scene_txt.contains("\"token\""), "written scene must author token");
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_eq!(scene.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(scene.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((scene.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert!(scene.record_contact_events);
    assert_eq!(scene.spawns[0].at_step, 30);
    assert_token(&scene.spawns[0].body);
    assert_eq!(scene.despawns[0].at_step, 80);
    assert_eq!(scene.despawns[0].body, "bar");
    assert_walker(body_by_id(&scene, "walker"));
    assert_bar(body_by_id(&scene, "bar"));
    assert_impulses(&scene.impulses);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.lights.len(), 2);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    let bodies = v["bodies"].as_array().unwrap();
    assert!(bodies.iter().any(|b| b["id"] == "token"), "written dump must include token");
    assert!(bodies.iter().all(|b| b["id"] != "bar"), "written dump must omit bar");
    let token = bodies.iter().find(|b| b["id"] == "token").unwrap();
    let tx = token["position"][0].as_f64().unwrap();
    let ty = token["position"][1].as_f64().unwrap();
    assert!((tx - 0.70).abs() < 0.08 && (ty - 0.12).abs() < 0.08);
    let spawned = v["spawned"].as_array().expect("written dump must have spawned");
    assert!(spawned.iter().any(|s| s["id"] == "token" && s["at_step"] == 30));
    let despawned = v["despawned"].as_array().expect("written dump must have despawned");
    assert!(despawned.iter().any(|s| s["id"] == "bar" && s["at_step"] == 80));
    let walker_state = bodies.iter().find(|b| b["id"] == "walker").expect("dump should record the walker");
    let wx = walker_state["position"][0].as_f64().unwrap();
    let wy = walker_state["position"][1].as_f64().unwrap();
    assert!(wx <= 1.15 - 0.4, "written dump walker.x should move >= 0.4 toward -X, got {wx}");
    assert!(wy >= 0.14 && wy <= 0.28, "written dump walker.y should be on the floor, got {wy}");
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}

#[test]
fn increment50_applies_spawn_despawn_at_step_start() {
    let scene = increment50_scene();
    let before_spawn = step_physics(&scene, 30, DEFAULT_DT).expect("30 steps");
    assert!(
        before_spawn.bodies.iter().all(|b| b.id != "token"),
        "token must not exist before at_step 30 (0-based i in 0..30 never hits 30)"
    );
    assert!(before_spawn.bodies.iter().any(|b| b.id == "bar"));
    let after_spawn = step_physics(&scene, 31, DEFAULT_DT).expect("31 steps");
    assert!(
        after_spawn.bodies.iter().any(|b| b.id == "token"),
        "token must exist after the start of step 30"
    );
    assert!(after_spawn.bodies.iter().any(|b| b.id == "bar"));
    let before_despawn = step_physics(&scene, 80, DEFAULT_DT).expect("80 steps");
    assert!(before_despawn.bodies.iter().any(|b| b.id == "bar"), "bar must remain before at_step 80");
    let after_despawn = step_physics(&scene, 81, DEFAULT_DT).expect("81 steps");
    assert!(
        after_despawn.bodies.iter().all(|b| b.id != "bar"),
        "bar must be gone after the start of step 80"
    );
    assert!(after_despawn.bodies.iter().any(|b| b.id == "token"));
}
