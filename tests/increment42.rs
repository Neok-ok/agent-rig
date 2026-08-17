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
    increment39_scene_json, increment40_scene_json, increment41_scene,
    increment41_scene_json, increment42_scene, increment42_scene_json,
    parse_scene, run_increment42, step_physics,
    Impulse, Joint, Light, Shape, DEFAULT_DT, INCREMENT42_STEPS,
};

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn assert_ball_impulse(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 1, "must keep exactly one impulse, got {}", impulses.len());
    assert_eq!(impulses[0].body, "ball");
    let lin = impulses[0].linear;
    assert!(
        (lin[0] - 1.8).abs() < 1e-5 && (lin[1] - 0.4).abs() < 1e-5 && (lin[2] - 0.5).abs() < 1e-5,
        "ball impulse linear should be [1.8, 0.4, 0.5], got {lin:?}"
    );
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
    assert!(
        !body.kinematic,
        "bob must not be kinematic, got kinematic={}",
        body.kinematic
    );
    match body.shape {
        Shape::Sphere { radius } => {
            assert!(
                (radius - 0.08).abs() < 1e-5,
                "bob sphere radius should be 0.08, got {radius}"
            );
        }
        _ => panic!("bob should be a sphere, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] - 0.35).abs() < 1e-5 && (p[1] - 0.88).abs() < 1e-5 && (p[2] - 1.75).abs() < 1e-5,
        "bob position should be [0.35, 0.88, 1.75], got {p:?}"
    );
    assert!(
        (body.mass - 0.2).abs() < 1e-5,
        "bob mass should be 0.2, got {}",
        body.mass
    );
    let a = body.material.albedo;
    assert!(
        (a[0] - 0.82).abs() < 1e-5 && (a[1] - 0.64).abs() < 1e-5 && (a[2] - 0.22).abs() < 1e-5,
        "bob albedo should be [0.82, 0.64, 0.22], got {a:?}"
    );
    assert!(
        (body.material.roughness - 0.28).abs() < 1e-5,
        "bob roughness should be 0.28, got {}",
        body.material.roughness
    );
    assert!(
        (body.material.metallic - 0.85).abs() < 1e-5,
        "bob metallic should be 0.85, got {}",
        body.material.metallic
    );
}

fn assert_no_bob(scene: &agent_rig::Scene, name: &str) {
    assert!(
        scene.bodies.iter().all(|b| b.id != "bob"),
        "{name} must stay bob-free"
    );
    for j in &scene.joints {
        assert!(
            !matches!(j, Joint::Distance { .. }),
            "{name} must stay distance-joint-free"
        );
    }
}

fn gate_hinge(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 3], [f32; 2], f32, f32) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            anchor,
            axis,
            limits,
            motor_target_velocity,
            motor_max_force,
        } = j
        {
            if body_a == "ground" && body_b == "gate" {
                let lim = limits.expect("ground–gate hinge must author limits");
                return (*anchor, *axis, lim, *motor_target_velocity, *motor_max_force);
            }
        }
    }
    panic!("scene missing ground–gate hinge");
}

fn distance_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], f32) {
    for j in &scene.joints {
        if let Joint::Distance {
            body_a,
            body_b,
            anchor,
            rest_length,
            ..
        } = j
        {
            return (body_a, body_b, *anchor, *rest_length);
        }
    }
    panic!("scene missing distance joint");
}

fn slider_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], [f32; 2], f32, f32) {
    for j in &scene.joints {
        if let Joint::Slider {
            body_a,
            body_b,
            axis,
            limits,
            motor_target_velocity,
            motor_max_force,
            ..
        } = j
        {
            return (
                body_a,
                body_b,
                *axis,
                *limits,
                *motor_target_velocity,
                *motor_max_force,
            );
        }
    }
    panic!("scene missing slider joint");
}

fn fixed_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3]) {
    for j in &scene.joints {
        if let Joint::Fixed {
            body_a,
            body_b,
            anchor,
        } = j
        {
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
fn increment42_does_not_mutate_prior_scene_json() {
    let prior_jsons: [(&str, &str); 24] = [
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
    ];
    let expected_bodies = [
        5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 8, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 12, 13, 14,
    ];
    for ((name, json), n) in prior_jsons.iter().zip(expected_bodies) {
        let parsed = parse_scene(json).expect(name);
        assert_eq!(
            parsed.bodies.len(),
            n,
            "increment {name} scene JSON must stay {n} bodies"
        );
        assert!(
            !json.contains("\"distance\""),
            "must not mutate increment {name} scene JSON with a distance joint"
        );
        assert!(
            !json.contains("\"bob\""),
            "must not mutate increment {name} scene JSON with a bob body"
        );
        assert_no_bob(&parsed, &format!("increment {name} JSON"));
    }

    let live41 = increment41_scene();
    assert_no_bob(&live41, "increment41_scene()");
    assert_gate(body_by_id(&live41, "gate"));
    assert_rider(body_by_id(&live41, "rider"));
    assert_platform(body_by_id(&live41, "platform"));
    let parsed41 = parse_scene(increment41_scene_json()).expect("inc41 live json");
    assert_no_bob(&parsed41, "increment41 JSON");
    assert_eq!(
        live41.bodies.len(),
        parsed41.bodies.len(),
        "increment41 JSON must stay unchanged vs increment41_scene()"
    );

    let live42 = increment42_scene();
    assert_eq!(
        live41.lights.len(),
        live42.lights.len(),
        "increment42 must not add lights[] entries vs increment 41"
    );
    assert_eq!(
        live41.bodies.len() + 1,
        live42.bodies.len(),
        "increment42 must add only the bob body vs increment 41"
    );
    assert_eq!(
        live41.joints.len() + 1,
        live42.joints.len(),
        "increment42 must add only the gate–bob distance joint vs increment 41"
    );
    assert_eq!(
        live41.impulses.len(),
        live42.impulses.len(),
        "increment42 must keep the ball impulse vs increment 41"
    );
    assert_eq!(
        live41.camera.position,
        live42.camera.position,
        "camera must stay increment-41"
    );
    assert_eq!(live41.camera.look_at, live42.camera.look_at);
    assert!(live42.bodies.iter().any(|b| b.id == "bob"));
    assert!(live42.joints.iter().any(|j| matches!(j, Joint::Distance { .. })));
}

#[test]
fn increment42_authors_the_bob() {
    let parsed = parse_scene(increment42_scene_json()).expect("increment42 JSON should parse");
    let parsed_bob = parsed
        .bodies
        .iter()
        .find(|b| b.id == "bob")
        .expect("increment42 JSON must author bob");
    assert_bob(parsed_bob);
    assert_gate(body_by_id(&parsed, "gate"));
    assert_rider(body_by_id(&parsed, "rider"));
    assert_platform(body_by_id(&parsed, "platform"));
    let (a, b, anchor, rest) = distance_of(&parsed);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5
            && (anchor[1] - 0.76).abs() < 1e-5
            && (anchor[2] - 1.75).abs() < 1e-5,
        "distance anchor should be [0.35, 0.76, 1.75], got {anchor:?}"
    );
    assert!(
        (rest - 0.38).abs() < 1e-5,
        "distance rest_length should be 0.38, got {rest}"
    );
    let (_anchor, _axis, limits, v, f) = gate_hinge(&parsed);
    assert!((limits[0] - 0.0).abs() < 1e-5 && (limits[1] - 1.15).abs() < 1e-5);
    assert!((v - 1.4).abs() < 1e-5 && (f - 5.0).abs() < 1e-5);
    assert_eq!(
        parsed.camera.position,
        increment41_scene().camera.position,
        "parsed camera must stay increment-41"
    );
    assert_eq!(parsed.camera.look_at, increment41_scene().camera.look_at);
    assert_ball_impulse(&parsed.impulses);

    let live = increment42_scene();
    assert_bob(body_by_id(&live, "bob"));
    assert_gate(body_by_id(&live, "gate"));
    assert_rider(body_by_id(&live, "rider"));
    assert_platform(body_by_id(&live, "platform"));
    let (a, b, anchor, rest) = distance_of(&live);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!(
        (anchor[0] - 0.35).abs() < 1e-5
            && (anchor[1] - 0.76).abs() < 1e-5
            && (anchor[2] - 1.75).abs() < 1e-5,
        "live distance anchor {anchor:?}"
    );
    assert!((rest - 0.38).abs() < 1e-5, "live rest_length {rest}");
    assert_eq!(live.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(live.camera.look_at, [0.1, 0.38, 0.0]);
    assert_ball_impulse(&live.impulses);

    let prior = increment41_scene();
    assert_no_bob(&prior, "increment41_scene()");
    assert_gate(body_by_id(&prior, "gate"));
}

#[test]
fn increment42_keeps_courtyard() {
    let scene = increment42_scene();
    assert!(
        scene.bodies.len() >= 15,
        "scene must have courtyard + gate + bob, got {}",
        scene.bodies.len()
    );
    let inc41 = increment41_scene();
    assert_eq!(
        scene.lights.len(),
        inc41.lights.len(),
        "no extra lights[] entries vs increment 41"
    );
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert_eq!(
        scene.bodies.len(),
        inc41.bodies.len() + 1,
        "only the bob body vs increment 41"
    );
    assert_eq!(
        scene.joints.len(),
        inc41.joints.len() + 1,
        "only the gate–bob distance joint vs increment 41"
    );
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.triggers.len(), 1);
    assert_eq!(scene.triggers[0].id, "drawer_open");
    assert_ball_impulse(&scene.impulses);
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
fn increment42_bob_drops_and_rope_holds() {
    let scene = increment42_scene();
    let dump = step_physics(&scene, INCREMENT42_STEPS, DEFAULT_DT).expect("physics");
    assert_ball_impulse(&dump.impulses);

    let _authored_bob = body_by_id(&scene, "bob");
    let bob = dump
        .bodies
        .iter()
        .find(|b| b.id == "bob")
        .expect("dump missing bob");
    assert!(!bob.kinematic, "dump bob must not be kinematic");
    assert!(
        bob.position[1] < 0.88 - 0.12,
        "bob should drop below authored 0.88 by > 0.12, got {:?}",
        bob.position
    );

    let authored_gate = body_by_id(&scene, "gate");
    let gate = dump
        .bodies
        .iter()
        .find(|b| b.id == "gate")
        .expect("dump missing gate");
    let (da, db, authored_anchor, rest) = distance_of(&scene);
    assert_eq!(da, "gate");
    assert_eq!(db, "bob");
    let current_anchor = current_world_anchor(
        authored_gate.position,
        authored_gate.rotation_wxyz,
        authored_anchor,
        gate.position,
        gate.rotation_wxyz,
    );
    let rope_len = dist3(current_anchor, bob.position);
    assert!(
        rope_len <= rest + 0.08,
        "rope should hold: |bob COM - current gate-top| <= rest_length+0.08 ({rest}+0.08), got {rope_len} anchor={current_anchor:?} bob={:?}",
        bob.position
    );

    let dist_j = dump
        .joints
        .iter()
        .find(|j| j.kind == "distance")
        .expect("dump missing distance joint");
    assert_eq!(dist_j.body_a, "gate");
    assert_eq!(dist_j.body_b, "bob");
    let dump_rest = dist_j
        .rest_length
        .expect("dump distance joint must record rest_length");
    assert!(
        (dump_rest - 0.38).abs() < 1e-5,
        "dump rest_length should be 0.38, got {dump_rest}"
    );
    assert!(
        (dist_j.anchor[0] - 0.35).abs() < 1e-5
            && (dist_j.anchor[1] - 0.76).abs() < 1e-5
            && (dist_j.anchor[2] - 1.75).abs() < 1e-5,
        "dump distance anchor should be the authored world gate-top, got {:?}",
        dist_j.anchor
    );

    let authored_x = body_by_id(&scene, "platform").position[0];
    let platform = dump
        .bodies
        .iter()
        .find(|b| b.id == "platform")
        .expect("dump missing platform");
    assert!(platform.kinematic, "dump platform must record kinematic: true");
    assert!(
        (platform.position[0] - authored_x).abs() > 0.4,
        "platform should slide +X by more than 0.4, authored_x={authored_x} got {:?}",
        platform.position
    );

    let authored_rider_x = body_by_id(&scene, "rider").position[0];
    let rider = dump
        .bodies
        .iter()
        .find(|b| b.id == "rider")
        .expect("dump missing rider");
    assert!(!rider.kinematic, "dump rider must not be kinematic");
    assert!(
        (rider.position[0] - authored_rider_x).abs() > 0.35,
        "rider should ride +X by more than 0.35, got {:?}",
        rider.position
    );
    assert!(
        rider.position[1] > 0.10,
        "rider COM should stay on the slab, got {:?}",
        rider.position
    );

    let gate_j = dump
        .joints
        .iter()
        .find(|j| j.kind == "hinge" && j.body_a == "ground" && j.body_b == "gate")
        .expect("dump missing ground–gate hinge");
    let glim = gate_j.limits.expect("dump gate hinge must record limits");
    assert!((glim[0] - 0.0).abs() < 1e-5 && (glim[1] - 1.15).abs() < 1e-5);
    let angle = gate_j.angle.expect("dump gate hinge must record angle");
    assert!(
        (angle - 1.15).abs() <= 0.2,
        "gate hinge angle should be within 0.2 of 1.15, got {angle}"
    );
    assert!(angle <= 1.30, "gate hinge angle must not pass 1.30, got {angle}");

    let ball = dump
        .bodies
        .iter()
        .find(|b| b.id == "ball")
        .expect("dump missing ball");
    assert!(
        (ball.position[0] + 1.1).abs() > 0.25,
        "gold ball COM should roll off the seat at x=-1.1, got {:?}",
        ball.position
    );

    let lid = dump
        .bodies
        .iter()
        .find(|b| b.id == "lid")
        .expect("dump missing lid");
    assert!(
        lid.position[1] > 0.20,
        "lid COM should stay on the crate, got {:?}",
        lid.position
    );

    let drawer = dump
        .bodies
        .iter()
        .find(|b| b.id == "drawer")
        .expect("dump missing drawer");
    assert!(
        drawer.position[2] < 1.15,
        "closed drawer COM should sit near z=1.02, got {:?}",
        drawer.position
    );

    let max_toi = scene.shapecasts[0].max_toi;
    let hit = dump
        .sweep_hits
        .iter()
        .find(|h| h.sweep == "drawer_sweep")
        .expect("dump sweep_hits must record drawer_sweep");
    assert_eq!(hit.body, "drawer");
    assert!(hit.toi > 0.0 && hit.toi < max_toi);
    let ray_hit = dump
        .ray_hits
        .iter()
        .find(|h| h.ray == "drawer_probe")
        .expect("dump ray_hits must still record drawer_probe");
    assert_eq!(ray_hit.body, "drawer");
}

#[test]
fn increment42_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment42.sh");
    assert!(script.is_file(), "scripts/increment42.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment42-threejs.sh");
    assert!(three.is_file(), "scripts/increment42-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment42-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment42(&out, INCREMENT42_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment42");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_bob(body_by_id(&scene, "bob"));
    assert_gate(body_by_id(&scene, "gate"));
    assert_rider(body_by_id(&scene, "rider"));
    assert_platform(body_by_id(&scene, "platform"));
    let (a, b, anchor, rest) = distance_of(&scene);
    assert_eq!(a, "gate");
    assert_eq!(b, "bob");
    assert!((anchor[1] - 0.76).abs() < 1e-5);
    assert!((rest - 0.38).abs() < 1e-5);
    assert_ball_impulse(&scene.impulses);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_eq!(scene.lights.len(), 2);

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 15);
    let bob_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "bob")
        .expect("dump should record the bob");
    assert_eq!(bob_state["kinematic"].as_bool().unwrap_or(false), false);
    let by = bob_state["position"][1].as_f64().unwrap();
    assert!(
        by < 0.88 - 0.12,
        "written dump bob y should drop below 0.76, got {by}"
    );
    let joints = v["joints"].as_array().expect("dump must have joints array");
    let dist = joints
        .iter()
        .find(|j| j["kind"] == "distance")
        .expect("dump joints must record the distance joint");
    assert_eq!(dist["body_a"], "gate");
    assert_eq!(dist["body_b"], "bob");
    assert!((dist["rest_length"].as_f64().unwrap() - 0.38).abs() < 1e-5);
    let gate_j = joints
        .iter()
        .find(|j| j["kind"] == "hinge" && j["body_a"] == "ground" && j["body_b"] == "gate")
        .expect("dump joints must record the ground–gate hinge");
    let gangle = gate_j["angle"].as_f64().expect("written dump gate hinge must record angle");
    assert!((gangle - 1.15).abs() <= 0.2);
    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
