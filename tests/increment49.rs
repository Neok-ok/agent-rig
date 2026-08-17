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
    increment48_scene, increment48_scene_json, increment49_scene,
    increment49_scene_json, parse_scene, run_increment49, step_physics, Impulse,
    Joint, Light, Shape, DEFAULT_DT, INCREMENT48_STEPS, INCREMENT49_STEPS,
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

#[test]
fn increment49_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 31] = [
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
    ];
    for (name, json) in prior_jsons {
        assert!(
            !json.contains("\"collision_groups\""),
            "must not mutate increment {name} scene JSON with collision_groups"
        );
        assert!(
            !json.contains("\"bar\""),
            "must not mutate increment {name} scene JSON with bar"
        );
    }

    let live48 = increment48_scene();
    let live49 = increment49_scene();
    assert!(
        live48.bodies.iter().all(|b| b.id != "bar" && b.collision_groups.is_default()),
        "increment48_scene must stay bar-free and group-free"
    );
    assert!(live48.record_contact_events);
    assert!(live49.record_contact_events, "increment49 must keep record_contact_events");
    assert_eq!(live48.lights.len(), live49.lights.len(), "increment49 must not add lights[] entries vs increment 48");
    assert_eq!(live48.bodies.len() + 1, live49.bodies.len(), "increment49 must add only bar vs increment 48");
    assert_eq!(live48.joints.len(), live49.joints.len(), "increment49 must not add joints vs increment 48");
    assert_eq!(live48.impulses.len(), live49.impulses.len());
    assert_eq!(live48.camera.position, live49.camera.position);
    assert_eq!(live48.camera.look_at, live49.camera.look_at);
    assert_eq!(live48.camera.fov_y_deg, live49.camera.fov_y_deg);
    let w48 = body_by_id(&live48, "walker");
    let w49 = body_by_id(&live49, "walker");
    assert_eq!(w48.position, w49.position, "increment49 must not change walker start");
    assert_eq!(
        w48.controller.as_ref().unwrap().desired_velocity,
        w49.controller.as_ref().unwrap().desired_velocity,
        "increment49 must not change walker controller"
    );
}

#[test]
fn increment49_scene_adds_bar_and_groups() {
    let parsed = parse_scene(increment49_scene_json()).expect("increment49 JSON should parse");
    assert_eq!(parsed.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(parsed.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((parsed.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert_no_extra_camera_keys(increment49_scene_json(), "increment49_scene_json");
    assert!(increment49_scene_json().contains("\"collision_groups\""));
    assert!(increment49_scene_json().contains("\"bar\""));
    assert_walker(body_by_id(&parsed, "walker"));
    assert_walker_groups(body_by_id(&parsed, "walker"));
    assert_bar(body_by_id(&parsed, "bar"));
    let (limits, vel, force, pos) = gate_hinge(&parsed);
    assert!((limits[0] - 0.0).abs() < 1e-5 && (limits[1] - 1.15).abs() < 1e-5);
    assert!(vel.abs() < 1e-5, "increment49 gate motor_target_velocity should stay 0, got {vel}");
    assert!((force - 5.0).abs() < 1e-5, "increment49 gate motor_max_force should stay 5.0, got {force}");
    let target = pos.expect("increment49 JSON gate must keep motor_target_position");
    assert!((target - 0.55).abs() < 1e-5, "increment49 JSON motor_target_position should stay 0.55, got {target}");
    assert_cork(body_by_id(&parsed, "cork"));
    assert_bob(body_by_id(&parsed, "bob"));
    assert_gate(body_by_id(&parsed, "gate"));
    assert_rider(body_by_id(&parsed, "rider"));
    assert_platform(body_by_id(&parsed, "platform"));
    assert_impulses(&parsed.impulses);

    let live = increment49_scene();
    assert_eq!(live.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(live.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((live.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert!(live.record_contact_events);
    assert_walker(body_by_id(&live, "walker"));
    assert_walker_groups(body_by_id(&live, "walker"));
    assert_bar(body_by_id(&live, "bar"));
    let (limits, vel, force, pos) = gate_hinge(&live);
    assert!((limits[0] - 0.0).abs() < 1e-5 && (limits[1] - 1.15).abs() < 1e-5);
    assert!(vel.abs() < 1e-5);
    assert!((force - 5.0).abs() < 1e-5);
    let target = pos.expect("live increment49 gate must keep motor_target_position");
    assert!((target - 0.55).abs() < 1e-5);
    assert_impulses(&live.impulses);

    let prior = increment48_scene();
    assert!(prior.bodies.iter().all(|b| b.id != "bar"));
    assert!(prior.bodies.iter().all(|b| b.collision_groups.is_default()));
}

#[test]
fn increment49_keeps_courtyard() {
    let scene = increment49_scene();
    let inc48 = increment48_scene();
    assert_eq!(scene.lights.len(), inc48.lights.len(), "no extra lights[] entries vs increment 48");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert_eq!(scene.bodies.len(), inc48.bodies.len() + 1, "only bar vs increment 48");
    assert_eq!(scene.joints.len(), inc48.joints.len(), "no new joints vs increment 48");
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
fn increment48_dump_has_no_collision_groups() {
    let scene = increment48_scene();
    assert!(scene.bodies.iter().all(|b| b.id != "bar"));
    assert!(scene.bodies.iter().all(|b| b.collision_groups.is_default()));
    let dump = step_physics(&scene, INCREMENT48_STEPS, DEFAULT_DT).expect("increment48 physics");
    assert!(
        dump.bodies.iter().all(|b| b.collision_groups.is_default()),
        "increment48 dump bodies must stay without authored groups"
    );
    let json = serde_json::to_value(&dump).expect("serialize increment48 dump");
    for body in json["bodies"].as_array().unwrap() {
        assert!(
            body.get("collision_groups").is_none(),
            "increment48 dump JSON must omit collision_groups, got {body}"
        );
    }
    let scene_json = serde_json::to_string(&scene).expect("serialize increment48 scene");
    assert!(
        !scene_json.contains("\"collision_groups\""),
        "increment48 scene JSON must omit collision_groups when default"
    );
    assert!(
        !scene_json.contains("\"bar\""),
        "increment48 scene JSON must not contain bar"
    );
}

#[test]
fn increment49_physics_walks_through_bar() {
    let scene = increment49_scene();
    let authored_x = body_by_id(&scene, "walker").position[0];
    let authored_bar = body_by_id(&scene, "bar").position;
    let dump = step_physics(&scene, INCREMENT49_STEPS, DEFAULT_DT).expect("physics");
    assert_impulses(&dump.impulses);

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
    assert_eq!(walker.collision_groups.membership, 2);
    assert_eq!(walker.collision_groups.filter, 1);

    let bar = dump.bodies.iter().find(|b| b.id == "bar").expect("dump missing bar");
    assert!(
        (bar.position[0] - authored_bar[0]).abs() < 1e-4
            && (bar.position[1] - authored_bar[1]).abs() < 1e-4
            && (bar.position[2] - authored_bar[2]).abs() < 1e-4,
        "bar should stay at authored pose {authored_bar:?}, got {:?}",
        bar.position
    );
    assert_eq!(bar.collision_groups.membership, 4);
    assert_eq!(bar.collision_groups.filter, 0xFFFF);

    let ctrl = dump
        .controllers
        .iter()
        .find(|c| c.id == "walker")
        .expect("dump.controllers must include walker");
    assert!(ctrl.grounded, "walker should be grounded after 120 steps");
    assert!(
        (ctrl.desired_velocity[0] + 0.55).abs() < 0.06
            && ctrl.desired_velocity[1].abs() < 1e-5
            && ctrl.desired_velocity[2].abs() < 1e-5,
        "dump desired_velocity should echo authored wish, got {:?}",
        ctrl.desired_velocity
    );
    assert!(
        ctrl.effective_translation.iter().any(|c| c.abs() > 1e-6),
        "last-step effective_translation should be non-zero, got {:?}",
        ctrl.effective_translation
    );

    assert!(
        !dump.contact_events.is_empty(),
        "increment49 dump must still record contact_events"
    );
    assert!(
        dump.contact_events.iter().any(|e| {
            e.kind == "started" && pair_involves(&e.body_a, &e.body_b, "bob", "ground")
        }),
        "contact_events must include started involving bob and ground, got {:?}",
        dump.contact_events
    );
    assert!(
        dump.contact_events.iter().any(|e| {
            e.kind == "started" && pair_involves(&e.body_a, &e.body_b, "rider", "platform")
        }),
        "contact_events must include started involving rider and platform, got {:?}",
        dump.contact_events
    );
    assert!(
        dump.contact_events.iter().all(|e| {
            !(e.kind == "started" && pair_involves(&e.body_a, &e.body_b, "walker", "bar"))
        }),
        "contact_events must NOT include a walker-bar started pair, got {:?}",
        dump.contact_events
    );

    let gate_j = dump
        .joints
        .iter()
        .find(|j| j.kind == "hinge" && j.body_a == "ground" && j.body_b == "gate")
        .expect("dump missing ground-gate hinge");
    let glim = gate_j.limits.expect("dump gate hinge must record limits");
    assert!((glim[0] - 0.0).abs() < 1e-5 && (glim[1] - 1.15).abs() < 1e-5);
    let target = gate_j
        .motor_target_position
        .expect("dump gate hinge must record motor_target_position");
    assert!((target - 0.55).abs() < 1e-5, "dump motor_target_position should be 0.55, got {target}");
    let angle = gate_j.angle.expect("dump gate hinge must record angle");
    assert!(
        (angle - 0.55).abs() <= 0.15,
        "gate hinge angle should be within 0.15 of 0.55, got {angle}"
    );
    assert!(
        angle >= 0.40 && angle <= 0.70,
        "gate hinge angle should be in [0.40, 0.70], got {angle}"
    );
    assert!(
        (angle - 1.15).abs() > 0.30,
        "gate must not park at the 1.15 limit, got {angle}"
    );

    let cork = dump.bodies.iter().find(|b| b.id == "cork").expect("dump missing cork");
    assert!(!cork.kinematic, "dump cork must not be kinematic");
    assert!(
        cork.position[1] < 1.15 - 0.15,
        "cork should drop below authored 1.15 by > 0.15, got {:?}",
        cork.position
    );

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

    let spring_j = dump
        .joints
        .iter()
        .find(|j| j.kind == "spring" && j.body_a == "gate" && j.body_b == "cork")
        .expect("dump missing gate-cork spring");
    assert!((spring_j.rest_length.expect("dump spring rest_length") - 0.42).abs() < 1e-5);
    assert!((spring_j.stiffness.expect("dump spring stiffness") - 40.0).abs() < 1e-5);
    assert!((spring_j.damping.expect("dump spring damping") - 4.0).abs() < 1e-5);

    let rider = dump.bodies.iter().find(|b| b.id == "rider").expect("dump missing rider");
    let platform = dump.bodies.iter().find(|b| b.id == "platform").expect("dump missing platform");
    assert!(platform.kinematic, "platform must stay kinematic velocity-based");
    assert!(rider.position[0] > -0.55 + 0.2, "rider should still ride +X, got {:?}", rider.position);

    assert!(dump.ray_hits.iter().any(|h| h.ray == "drawer_probe"));
    assert!(dump.sweep_hits.iter().any(|h| h.sweep == "drawer_sweep"));
}

#[test]
fn increment49_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment49.sh");
    assert!(script.is_file(), "scripts/increment49.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment49-threejs.sh");
    assert!(three.is_file(), "scripts/increment49-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment49-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment49(&out, INCREMENT49_STEPS, DEFAULT_DT, 200, 112).expect("run_increment49");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");
    assert!(
        !out.join("frame_01.png").exists() && !out.join("cameras.json").exists(),
        "must not write a second frame or cameras file"
    );

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert_no_extra_camera_keys(&scene_txt, "written scene.json");
    assert!(scene_txt.contains("\"collision_groups\""), "written scene must author collision_groups");
    assert!(scene_txt.contains("\"bar\""), "written scene must author bar");
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_eq!(scene.camera.position, [1.85, 1.35, 3.15]);
    assert_eq!(scene.camera.look_at, [0.35, 0.42, 1.55]);
    assert!((scene.camera.fov_y_deg - 40.0).abs() < 1e-5);
    assert!(scene.record_contact_events, "written scene must keep contact events");
    assert_walker(body_by_id(&scene, "walker"));
    assert_walker_groups(body_by_id(&scene, "walker"));
    assert_bar(body_by_id(&scene, "bar"));
    let (_lim, vel, force, pos) = gate_hinge(&scene);
    assert!(vel.abs() < 1e-5);
    assert!((force - 5.0).abs() < 1e-5);
    let target = pos.expect("written scene gate must author motor_target_position");
    assert!((target - 0.55).abs() < 1e-5);
    assert_cork(body_by_id(&scene, "cork"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_impulses(&scene.impulses);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.lights.len(), 2);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 18);
    let controllers = v["controllers"].as_array().expect("written dump must have controllers");
    let wctrl = controllers
        .iter()
        .find(|c| c["id"] == "walker")
        .expect("written dump controllers must include walker");
    assert_eq!(wctrl["grounded"], true);
    let walker_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "walker")
        .expect("dump should record the walker");
    let wx = walker_state["position"][0].as_f64().unwrap();
    let wy = walker_state["position"][1].as_f64().unwrap();
    assert!(wx <= 1.15 - 0.4, "written dump walker.x should move ≥ 0.4 toward -X, got {wx}");
    assert!(wy >= 0.14 && wy <= 0.28, "written dump walker.y should be on the floor, got {wy}");
    assert_eq!(walker_state["collision_groups"]["membership"], 2);
    assert_eq!(walker_state["collision_groups"]["filter"], 1);
    let bar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "bar")
        .expect("dump should record the bar");
    let bx = bar_state["position"][0].as_f64().unwrap();
    let by = bar_state["position"][1].as_f64().unwrap();
    let bz = bar_state["position"][2].as_f64().unwrap();
    assert!((bx - 0.55).abs() < 1e-3 && (by - 0.22).abs() < 1e-3 && (bz - 1.45).abs() < 1e-3);
    assert_eq!(bar_state["collision_groups"]["membership"], 4);
    assert_eq!(bar_state["collision_groups"]["filter"], 65535);
    let events = v["contact_events"].as_array().expect("written dump must have contact_events");
    assert!(
        events.iter().any(|e| {
            e["kind"] == "started"
                && pair_involves(
                    e["body_a"].as_str().unwrap_or(""),
                    e["body_b"].as_str().unwrap_or(""),
                    "bob",
                    "ground",
                )
        }),
        "written dump contact_events must include started bob+ground, got {events:?}"
    );
    assert!(
        events.iter().all(|e| {
            !(e["kind"] == "started"
                && pair_involves(
                    e["body_a"].as_str().unwrap_or(""),
                    e["body_b"].as_str().unwrap_or(""),
                    "walker",
                    "bar",
                ))
        }),
        "written dump contact_events must not include walker-bar started, got {events:?}"
    );
    let joints = v["joints"].as_array().expect("dump must have joints array");
    let gate_j = joints
        .iter()
        .find(|j| j["kind"] == "hinge" && j["body_a"] == "ground" && j["body_b"] == "gate")
        .expect("dump joints must record the ground-gate hinge");
    let gangle = gate_j["angle"].as_f64().expect("written dump gate hinge must record angle");
    assert!((gangle - 0.55).abs() <= 0.15, "written dump gate angle should be ~0.55, got {gangle}");
    assert!((gangle - 1.15).abs() > 0.30, "written dump gate must not park at 1.15, got {gangle}");
    let bob_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "bob")
        .expect("dump should record the bob");
    let by = bob_state["position"][1].as_f64().unwrap();
    assert!(by < 0.22, "written dump bob y should be on the bowl (< 0.22), got {by}");
    let broken = v["broken_joints"].as_array().expect("dump must record broken_joints");
    assert!(
        broken.iter().any(|j| j["kind"] == "distance" && j["body_a"] == "gate" && j["body_b"] == "bob"),
        "written dump broken_joints must include gate-bob distance, got {broken:?}"
    );
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
