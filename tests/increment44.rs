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
    increment42_scene_json, increment43_scene, increment43_scene_json,
    increment44_scene, increment44_scene_json, parse_scene, run_increment44,
    step_physics, Impulse, Joint, Light, Shape, DEFAULT_DT, INCREMENT44_STEPS,
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

fn assert_increment44_impulses(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 2, "increment44 must keep ball + bob impulses, got {}", impulses.len());
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

fn assert_cork(body: &agent_rig::Body) {
    assert_eq!(body.id, "cork");
    assert!(
        !body.kinematic,
        "cork must not be kinematic, got kinematic={}",
        body.kinematic
    );
    match body.shape {
        Shape::Sphere { radius } => {
            assert!(
                (radius - 0.14).abs() < 1e-5,
                "cork sphere radius should be 0.14, got {radius}"
            );
        }
        _ => panic!("cork should be a sphere, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] - 0.35).abs() < 1e-5 && (p[1] - 1.15).abs() < 1e-5 && (p[2] - 1.75).abs() < 1e-5,
        "cork position should be [0.35, 1.15, 1.75], got {p:?}"
    );
    assert!(
        (body.mass - 0.25).abs() < 1e-5,
        "cork mass should be 0.25, got {}",
        body.mass
    );
    let a = body.material.albedo;
    assert!(
        (a[0] - 0.72).abs() < 1e-5 && (a[1] - 0.58).abs() < 1e-5 && (a[2] - 0.32).abs() < 1e-5,
        "cork albedo should be [0.72, 0.58, 0.32], got {a:?}"
    );
    assert!(
        (body.material.roughness - 0.8).abs() < 1e-5,
        "cork roughness should be 0.8, got {}",
        body.material.roughness
    );
    assert!(
        body.material.metallic.abs() < 1e-5,
        "cork metallic should be 0, got {}",
        body.material.metallic
    );
}

fn assert_no_cork(scene: &agent_rig::Scene, name: &str) {
    assert!(
        scene.bodies.iter().all(|b| b.id != "cork"),
        "{name} must stay cork-free"
    );
    for j in &scene.joints {
        assert!(
            !matches!(j, Joint::Spring { .. }),
            "{name} must stay spring-free"
        );
    }
}

fn gate_hinge(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 3], [f32; 2], f32, f32) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a, body_b, anchor, axis, limits, motor_target_velocity, motor_max_force, ..
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

fn spring_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], f32, f32, f32) {
    for j in &scene.joints {
        if let Joint::Spring {
            body_a, body_b, anchor, rest_length, stiffness, damping,
        } = j
        {
            return (body_a, body_b, *anchor, *rest_length, *stiffness, *damping);
        }
    }
    panic!("scene missing spring joint");
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

fn quat_rotate_wxyz(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    let ux = y * v[2] - z * v[1];
    let uy = z * v[0] - x * v[2];
    let uz = x * v[1] - y * v[0];
    let uux = y * uz - z * uy;
    let uuy = z * ux - x * uz;
    let uuz = x * uy - y * ux;
    [
        v[0] + 2.0 * (w * ux + uux),
        v[1] + 2.0 * (w * uy + uuy),
        v[2] + 2.0 * (w * uz + uuz),
    ]
}

fn current_world_anchor(
    authored_body_pos: [f32; 3],
    authored_body_rot: [f32; 4],
    authored_anchor: [f32; 3],
    body_pos: [f32; 3],
    body_rot: [f32; 4],
) -> [f32; 3] {
    let world_off = [
        authored_anchor[0] - authored_body_pos[0],
        authored_anchor[1] - authored_body_pos[1],
        authored_anchor[2] - authored_body_pos[2],
    ];
    let inv = [
        authored_body_rot[0],
        -authored_body_rot[1],
        -authored_body_rot[2],
        -authored_body_rot[3],
    ];
    let local = quat_rotate_wxyz(inv, world_off);
    let r = quat_rotate_wxyz(body_rot, local);
    [
        body_pos[0] + r[0],
        body_pos[1] + r[1],
        body_pos[2] + r[2],
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[test]
fn increment44_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 26] = [
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
    ];
    for (name, json) in prior_jsons {
        assert!(!json.contains("\"spring\""), "must not mutate increment {name} scene JSON with a spring joint");
        assert!(!json.contains("\"cork\""), "must not mutate increment {name} scene JSON with a cork body");
    }

    let live43 = increment43_scene();
    assert_no_cork(&live43, "increment43_scene()");
    let live44 = increment44_scene();
    assert_eq!(live43.lights.len(), live44.lights.len(), "increment44 must not add lights[] entries vs increment 43");
    assert_eq!(live43.bodies.len() + 1, live44.bodies.len(), "increment44 must add only the cork body vs increment 43");
    assert_eq!(live43.joints.len() + 1, live44.joints.len(), "increment44 must add only the gate-cork spring vs increment 43");
    assert_eq!(live43.impulses.len(), live44.impulses.len(), "increment44 must keep ball + bob impulses vs increment 43");
    assert_eq!(live43.camera.position, live44.camera.position, "camera must stay increment-43");
    assert_eq!(live43.camera.look_at, live44.camera.look_at);
}

#[test]
fn increment44_authors_cork_and_spring() {
    let parsed = parse_scene(increment44_scene_json()).expect("increment44 JSON should parse");
    assert_cork(body_by_id(&parsed, "cork"));
    assert_bob(body_by_id(&parsed, "bob"));
    assert_gate(body_by_id(&parsed, "gate"));
    assert_rider(body_by_id(&parsed, "rider"));
    assert_platform(body_by_id(&parsed, "platform"));
    let (a, b, anchor, rest, stiff, damp) = spring_of(&parsed);
    assert_eq!(a, "gate");
    assert_eq!(b, "cork");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5 && (anchor[1] - 0.76).abs() < 1e-5 && (anchor[2] - 1.75).abs() < 1e-5,
        "spring anchor should be [0.35, 0.76, 1.75], got {anchor:?}"
    );
    assert!((rest - 0.42).abs() < 1e-5, "spring rest_length should be 0.42, got {rest}");
    assert!((stiff - 40.0).abs() < 1e-5, "spring stiffness should be 40, got {stiff}");
    assert!((damp - 4.0).abs() < 1e-5, "spring damping should be 4, got {damp}");
    assert_increment44_impulses(&parsed.impulses);
    assert_eq!(parsed.camera.position, increment43_scene().camera.position);
    assert_eq!(parsed.camera.look_at, increment43_scene().camera.look_at);

    let live = increment44_scene();
    assert_cork(body_by_id(&live, "cork"));
    assert_bob(body_by_id(&live, "bob"));
    assert_gate(body_by_id(&live, "gate"));
    assert_rider(body_by_id(&live, "rider"));
    assert_platform(body_by_id(&live, "platform"));
    let (a, b, anchor, rest, stiff, damp) = spring_of(&live);
    assert_eq!(a, "gate");
    assert_eq!(b, "cork");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5 && (anchor[1] - 0.76).abs() < 1e-5 && (anchor[2] - 1.75).abs() < 1e-5,
        "live spring anchor {anchor:?}"
    );
    assert!((rest - 0.42).abs() < 1e-5, "live rest_length {rest}");
    assert!((stiff - 40.0).abs() < 1e-5, "live stiffness {stiff}");
    assert!((damp - 4.0).abs() < 1e-5, "live damping {damp}");
    assert_eq!(live.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(live.camera.look_at, [0.1, 0.38, 0.0]);
    assert_increment44_impulses(&live.impulses);

    let prior = increment43_scene();
    assert_no_cork(&prior, "increment43_scene()");
    let ser43 = serde_json::to_string(&prior).expect("serialize increment43_scene");
    assert!(!ser43.contains("\"spring\""), "increment43_scene() must serialize without spring");
    assert!(!ser43.contains("\"cork\""), "increment43_scene() must serialize without cork");
}

#[test]
fn increment44_keeps_courtyard() {
    let scene = increment44_scene();
    assert!(scene.bodies.len() >= 16, "scene must have courtyard + gate + bob + cork, got {}", scene.bodies.len());
    let inc43 = increment43_scene();
    assert_eq!(scene.lights.len(), inc43.lights.len(), "no extra lights[] entries vs increment 43");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert_eq!(scene.bodies.len(), inc43.bodies.len() + 1, "only the cork body vs increment 43");
    assert_eq!(scene.joints.len(), inc43.joints.len() + 1, "only the spring joint vs increment 43");
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.triggers.len(), 1);
    assert_eq!(scene.triggers[0].id, "drawer_open");
    assert_increment44_impulses(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_cork(body_by_id(&scene, "cork"));

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
    let (da, db, _danchor, drest, dbrk) = distance_of(&scene);
    assert_eq!(da, "gate");
    assert_eq!(db, "bob");
    assert!((drest - 0.38).abs() < 1e-5);
    assert!((dbrk - 1.5).abs() < 0.35);
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
fn increment43_scene_stays_cork_free_and_spring_free() {
    let scene = increment43_scene();
    assert_no_cork(&scene, "increment43_scene()");
    assert_bob(body_by_id(&scene, "bob"));
    assert!(scene.joints.iter().any(|j| matches!(j, Joint::Distance { .. })));
    assert!(scene.joints.iter().all(|j| !matches!(j, Joint::Spring { .. })));
}

#[test]
fn increment44_cork_settles_on_spring_and_bob_still_falls() {
    let scene = increment44_scene();
    let dump = step_physics(&scene, INCREMENT44_STEPS, DEFAULT_DT).expect("physics");
    assert_increment44_impulses(&dump.impulses);

    let cork = dump.bodies.iter().find(|b| b.id == "cork").expect("dump missing cork");
    assert!(!cork.kinematic, "dump cork must not be kinematic");
    assert!(
        cork.position[1] < 1.15 - 0.15,
        "cork should drop below authored 1.15 by > 0.15, got {:?}",
        cork.position
    );

    let authored_gate = body_by_id(&scene, "gate");
    let gate = dump.bodies.iter().find(|b| b.id == "gate").expect("dump missing gate");
    let (sa, sb, authored_anchor, rest, stiff, damp) = spring_of(&scene);
    assert_eq!(sa, "gate");
    assert_eq!(sb, "cork");
    assert!((rest - 0.42).abs() < 1e-5);
    assert!((stiff - 40.0).abs() < 1e-5);
    assert!((damp - 4.0).abs() < 1e-5);
    let current_anchor = current_world_anchor(
        authored_gate.position,
        authored_gate.rotation_wxyz,
        authored_anchor,
        gate.position,
        gate.rotation_wxyz,
    );
    let spring_len = dist3(current_anchor, cork.position);
    assert!(
        (spring_len - rest).abs() <= 0.12,
        "spring should settle: |cork COM - current gate-top| within 0.12 of rest_length {rest}, got {spring_len} anchor={current_anchor:?} cork={:?}",
        cork.position
    );

    let spring_j = dump.joints.iter().find(|j| j.kind == "spring" && j.body_a == "gate" && j.body_b == "cork")
        .expect("dump missing gate-cork spring");
    let dump_rest = spring_j.rest_length.expect("dump spring must record rest_length");
    assert!((dump_rest - 0.42).abs() < 1e-5);
    let dump_stiff = spring_j.stiffness.expect("dump spring must record stiffness");
    assert!((dump_stiff - 40.0).abs() < 1e-5);
    let dump_damp = spring_j.damping.expect("dump spring must record damping");
    assert!((dump_damp - 4.0).abs() < 1e-5);

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
fn increment44_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment44.sh");
    assert!(script.is_file(), "scripts/increment44.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment44-threejs.sh");
    assert!(three.is_file(), "scripts/increment44-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment44-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment44(&out, INCREMENT44_STEPS, DEFAULT_DT, 200, 112).expect("run_increment44");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_cork(body_by_id(&scene, "cork"));
    assert_bob(body_by_id(&scene, "bob"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_platform(body_by_id(&scene, "platform"));
    let (a, b, _anchor, rest, stiff, damp) = spring_of(&scene);
    assert_eq!(a, "gate");
    assert_eq!(b, "cork");
    assert!((rest - 0.42).abs() < 1e-5);
    assert!((stiff - 40.0).abs() < 1e-5);
    assert!((damp - 4.0).abs() < 1e-5);
    assert_increment44_impulses(&scene.impulses);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.lights.len(), 2);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 16);
    let cork_state = v["bodies"].as_array().unwrap().iter().find(|b| b["id"] == "cork").expect("dump should record the cork");
    assert_eq!(cork_state["kinematic"].as_bool().unwrap_or(false), false);
    let cy = cork_state["position"][1].as_f64().unwrap();
    assert!(cy < 1.15 - 0.15, "written dump cork y should drop below 1.00, got {cy}");
    let bob_state = v["bodies"].as_array().unwrap().iter().find(|b| b["id"] == "bob").expect("dump should record the bob");
    let by = bob_state["position"][1].as_f64().unwrap();
    assert!(by < 0.22, "written dump bob y should be on the bowl (< 0.22), got {by}");
    let joints = v["joints"].as_array().expect("dump must have joints array");
    let spring = joints.iter().find(|j| j["kind"] == "spring").expect("dump joints must record the spring");
    assert_eq!(spring["body_a"], "gate");
    assert_eq!(spring["body_b"], "cork");
    assert!((spring["rest_length"].as_f64().unwrap() - 0.42).abs() < 1e-5);
    assert!((spring["stiffness"].as_f64().unwrap() - 40.0).abs() < 1e-5);
    assert!((spring["damping"].as_f64().unwrap() - 4.0).abs() < 1e-5);
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
