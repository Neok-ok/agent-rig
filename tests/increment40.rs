use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment19_scene_json, increment20_scene, increment20_scene_json,
    increment21_scene, increment21_scene_json, increment22_scene, increment22_scene_json,
    increment23_scene, increment23_scene_json, increment24_scene, increment24_scene_json,
    increment25_scene, increment25_scene_json, increment26_scene, increment26_scene_json,
    increment27_scene, increment27_scene_json, increment28_scene, increment28_scene_json,
    increment29_scene, increment29_scene_json, increment30_scene, increment30_scene_json,
    increment31_scene, increment31_scene_json, increment32_scene, increment32_scene_json,
    increment33_scene, increment33_scene_json, increment34_scene, increment34_scene_json,
    increment35_scene, increment35_scene_json, increment36_scene, increment36_scene_json,
    increment37_scene, increment37_scene_json, increment38_scene, increment38_scene_json,
    increment39_scene, increment39_scene_json, increment40_scene, increment40_scene_json,
    parse_scene, run_increment40, step_physics,
    Impulse, Joint, Light, Shape, DEFAULT_DT, INCREMENT40_STEPS,
};

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn pane_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pane.gltf")
}

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

fn area_light(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 2], [f32; 3], f32, [f32; 3]) {
    for light in &scene.lights {
        if let Light::Area {
            position,
            size,
            color,
            intensity,
            normal,
        } = light
        {
            return (*position, *size, *color, *intensity, *normal);
        }
    }
    panic!("scene missing area light");
}

fn pane_ior_and_transmission(mat: &serde_json::Value) -> (f64, f64) {
    let mut transmission = 0.0;
    let mut ior = mat["ior"].as_f64().unwrap_or(0.0);
    if let Some(ext) = mat.get("extensions") {
        if let Some(n) = ext
            .get("KHR_materials_transmission")
            .and_then(|t| t.get("transmissionFactor"))
            .and_then(|v| v.as_f64())
        {
            transmission = n;
        }
        if let Some(n) = ext
            .get("KHR_materials_ior")
            .and_then(|t| t.get("ior"))
            .and_then(|v| v.as_f64())
        {
            ior = n;
        }
    }
    (ior, transmission)
}

fn hinge_motor(scene: &agent_rig::Scene) -> (f32, f32) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            motor_target_velocity,
            motor_max_force,
            ..
        } = j
        {
            if body_a == "pillar" && body_b == "lantern" {
                return (*motor_target_velocity, *motor_max_force);
            }
        }
    }
    panic!("scene missing pillar–lantern hinge");
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

fn ball_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3]) {
    for j in &scene.joints {
        if let Joint::Ball {
            body_a,
            body_b,
            anchor,
        } = j
        {
            return (body_a, body_b, *anchor);
        }
    }
    panic!("scene missing ball joint");
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

fn lantern_of(scene: &agent_rig::Scene) -> &agent_rig::Body {
    body_by_id(scene, "lantern")
}

fn assert_ball_impulse(impulses: &[Impulse]) {
    assert_eq!(impulses.len(), 1, "increment40 must keep exactly one impulse, got {}", impulses.len());
    assert_eq!(impulses[0].body, "ball");
    let lin = impulses[0].linear;
    assert!(
        (lin[0] - 1.8).abs() < 1e-5 && (lin[1] - 0.4).abs() < 1e-5 && (lin[2] - 0.5).abs() < 1e-5,
        "ball impulse linear should be [1.8, 0.4, 0.5], got {lin:?}"
    );
}

fn assert_platform(body: &agent_rig::Body) {
    assert_eq!(body.id, "platform");
    assert!(
        body.kinematic,
        "platform must be kinematic, got kinematic={}",
        body.kinematic
    );
    let vel = body.linear_velocity;
    assert!(
        (vel[0] - 0.45).abs() < 1e-5 && (vel[1] - 0.0).abs() < 1e-5 && (vel[2] - 0.0).abs() < 1e-5,
        "platform linear_velocity should be [0.45, 0.0, 0.0], got {vel:?}"
    );
    match body.shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.55).abs() < 1e-5
                    && (size[1] - 0.06).abs() < 1e-5
                    && (size[2] - 0.35).abs() < 1e-5,
                "platform box size should be [0.55, 0.06, 0.35], got {size:?}"
            );
        }
        _ => panic!("platform should be a box, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] + 0.55).abs() < 1e-5 && (p[1] - 0.04).abs() < 1e-5 && (p[2] + 0.55).abs() < 1e-5,
        "platform position should be [-0.55, 0.04, -0.55], got {p:?}"
    );
    let a = body.material.albedo;
    assert!(
        (a[0] - 0.38).abs() < 1e-5 && (a[1] - 0.40).abs() < 1e-5 && (a[2] - 0.44).abs() < 1e-5,
        "platform albedo should be [0.38, 0.40, 0.44], got {a:?}"
    );
    assert!(
        (body.material.roughness - 0.55).abs() < 1e-5,
        "platform roughness should be 0.55, got {}",
        body.material.roughness
    );
}

fn assert_rider(body: &agent_rig::Body) {
    assert_eq!(body.id, "rider");
    assert!(
        !body.kinematic,
        "rider must not be kinematic, got kinematic={}",
        body.kinematic
    );
    match body.shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.16).abs() < 1e-5
                    && (size[1] - 0.16).abs() < 1e-5
                    && (size[2] - 0.16).abs() < 1e-5,
                "rider box size should be [0.16, 0.16, 0.16], got {size:?}"
            );
        }
        _ => panic!("rider should be a box, got {:?}", body.shape),
    }
    let p = body.position;
    assert!(
        (p[0] + 0.55).abs() < 1e-5 && (p[1] - 0.15).abs() < 1e-5 && (p[2] + 0.55).abs() < 1e-5,
        "rider position should be [-0.55, 0.15, -0.55], got {p:?}"
    );
    assert!(
        (body.mass - 0.35).abs() < 1e-5,
        "rider mass should be 0.35, got {}",
        body.mass
    );
    let a = body.material.albedo;
    assert!(
        (a[0] - 0.72).abs() < 1e-5 && (a[1] - 0.38).abs() < 1e-5 && (a[2] - 0.22).abs() < 1e-5,
        "rider albedo should be [0.72, 0.38, 0.22], got {a:?}"
    );
    assert!(
        (body.material.roughness - 0.7).abs() < 1e-5,
        "rider roughness should be 0.7, got {}",
        body.material.roughness
    );
    assert!(
        body.material.metallic.abs() < 1e-5,
        "rider metalness should be 0, got {}",
        body.material.metallic
    );
}

fn assert_no_rider(scene: &agent_rig::Scene, name: &str) {
    assert!(
        scene.bodies.iter().all(|b| b.id != "rider"),
        "{name} must stay rider-free"
    );
}

fn assert_no_platform_or_kinematic(scene: &agent_rig::Scene, name: &str) {
    assert!(
        scene.bodies.iter().all(|b| b.id != "platform"),
        "{name} must stay platform-free"
    );
    assert!(
        scene.bodies.iter().all(|b| !b.kinematic),
        "{name} must stay kinematic-free"
    );
}

#[test]
fn increment40_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
    let s19 = parse_scene(increment19_scene_json()).expect("inc19");
    let s20 = parse_scene(increment20_scene_json()).expect("inc20");
    let s21 = parse_scene(increment21_scene_json()).expect("inc21");
    let s22 = parse_scene(increment22_scene_json()).expect("inc22");
    let s23 = parse_scene(increment23_scene_json()).expect("inc23");
    let s24 = parse_scene(increment24_scene_json()).expect("inc24");
    let s25 = parse_scene(increment25_scene_json()).expect("inc25");
    let s26 = parse_scene(increment26_scene_json()).expect("inc26");
    let s27 = parse_scene(increment27_scene_json()).expect("inc27");
    let s28 = parse_scene(increment28_scene_json()).expect("inc28");
    let s29 = parse_scene(increment29_scene_json()).expect("inc29");
    let s30 = parse_scene(increment30_scene_json()).expect("inc30");
    let s31 = parse_scene(increment31_scene_json()).expect("inc31");
    let s32 = parse_scene(increment32_scene_json()).expect("inc32");
    let s33 = parse_scene(increment33_scene_json()).expect("inc33");
    let s34 = parse_scene(increment34_scene_json()).expect("inc34");
    let s35 = parse_scene(increment35_scene_json()).expect("inc35");
    let s36 = parse_scene(increment36_scene_json()).expect("inc36");
    let s37 = parse_scene(increment37_scene_json()).expect("inc37");
    let s38 = parse_scene(increment38_scene_json()).expect("inc38");
    let s39 = parse_scene(increment39_scene_json()).expect("inc39");
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
    assert_eq!(s19.bodies.len(), 5, "increment 19 scene JSON must stay 5 bodies");
    assert_eq!(s20.bodies.len(), 5, "increment 20 scene JSON must stay 5 bodies");
    assert_eq!(s21.bodies.len(), 7, "increment 21 scene JSON must stay 7 bodies");
    assert_eq!(s22.bodies.len(), 7, "increment 22 scene JSON must stay 7 bodies");
    assert_eq!(s23.bodies.len(), 7, "increment 23 scene JSON must stay 7 bodies");
    assert_eq!(s24.bodies.len(), 7, "increment 24 scene JSON must stay 7 bodies");
    assert_eq!(s25.bodies.len(), 7, "increment 25 scene JSON must stay 7 bodies");
    assert_eq!(s26.bodies.len(), 7, "increment 26 scene JSON must stay 7 bodies");
    assert_eq!(s27.bodies.len(), 7, "increment 27 scene JSON must stay 7 bodies");
    assert_eq!(s28.bodies.len(), 8, "increment 28 scene JSON must stay 8 bodies");
    assert_eq!(s29.bodies.len(), 9, "increment 29 scene JSON must stay 9 bodies");
    assert_eq!(s30.bodies.len(), 9, "increment 30 scene JSON must stay 9 bodies");
    assert_eq!(s31.bodies.len(), 10, "increment 31 scene JSON must stay 10 bodies");
    assert_eq!(s32.bodies.len(), 10, "increment 32 scene JSON must stay 10 bodies");
    assert_eq!(s33.bodies.len(), 10, "increment 33 scene JSON must stay 10 bodies");
    assert_eq!(s34.bodies.len(), 10, "increment 34 scene JSON must stay 10 bodies");
    assert_eq!(s35.bodies.len(), 10, "increment 35 scene JSON must stay 10 bodies");
    assert_eq!(s36.bodies.len(), 10, "increment 36 scene JSON must stay 10 bodies");
    assert_eq!(s37.bodies.len(), 11, "increment 37 scene JSON must stay 11 bodies");
    assert_eq!(s38.bodies.len(), 11, "increment 38 scene JSON must stay 11 bodies");
    assert_eq!(s39.bodies.len(), 12, "increment 39 scene JSON must stay 12 bodies");
    for (name, json) in [
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
    ] {
        assert!(
            !json.contains("\"rider\""),
            "must not mutate increment {name} scene JSON with a rider body"
        );
        let parsed = parse_scene(json).expect(name);
        assert_no_rider(&parsed, &format!("increment {name} JSON"));
    }

    assert_ball_impulse(&s38.impulses);
    assert!(s38.bodies.iter().any(|b| b.id == "lid"), "increment38 JSON must keep the lid");
    assert!(
        s38.joints.iter().any(|j| matches!(j, Joint::Fixed { .. })),
        "increment38 JSON must keep the fixed joint"
    );
    assert_eq!(s38.shapecasts.len(), 1, "increment38 JSON must keep drawer_sweep");
    assert_eq!(s38.shapecasts[0].id, "drawer_sweep");
    assert_eq!(s38.raycasts.len(), 1, "increment38 JSON must keep drawer_probe");
    assert_eq!(s38.raycasts[0].id, "drawer_probe");

    let live38 = increment38_scene();
    assert_no_platform_or_kinematic(&live38, "increment38_scene()");
    assert_ball_impulse(&live38.impulses);
    assert!(live38.bodies.iter().any(|b| b.id == "lid"));
    assert!(live38.joints.iter().any(|j| matches!(j, Joint::Fixed { .. })));
    assert_eq!(live38.shapecasts.len(), 1);
    assert_eq!(live38.shapecasts[0].id, "drawer_sweep");
    assert_eq!(live38.raycasts.len(), 1);
    assert_eq!(live38.raycasts[0].id, "drawer_probe");
    let (_a, _b, _axis, _limits, v38, f38) = slider_of(&live38);
    assert!(
        (v38 - (-2.0)).abs() < 1e-5 && (f38 - 6.0).abs() < 1e-5,
        "increment38 slider motor must stay -2/6, got {v38}/{f38}"
    );
    let lantern38 = lantern_of(&live38);
    assert!(
        (lantern38.material.emissive_intensity - 16.0).abs() < 1e-5,
        "increment38 lantern must stay intensity 16, got {}",
        lantern38.material.emissive_intensity
    );
    assert_eq!(live38.triggers.len(), 1);
    assert_eq!(live38.triggers[0].id, "drawer_open");
    assert!(live38.bodies.iter().any(|b| b.id == "charm"));
    assert!(live38.joints.iter().any(|j| matches!(j, Joint::Ball { .. })));
    let (hv, hf) = hinge_motor(&live38);
    assert!(
        (hv - 4.0).abs() < 1e-5 && (hf - 8.0).abs() < 1e-5,
        "increment38 hinge motor must stay 4/8, got {hv}/{hf}"
    );
    let live39 = increment39_scene();
    assert_no_rider(&live39, "increment39_scene()");
    assert!(live39.bodies.iter().any(|b| b.id == "platform"));
    assert_eq!(
        live39.bodies.iter().filter(|b| b.kinematic).count(),
        1,
        "increment39 must keep exactly one kinematic body"
    );
    assert_platform(body_by_id(&live39, "platform"));
    let parsed39 = parse_scene(increment39_scene_json()).expect("inc39 live json");
    assert_no_rider(&parsed39, "increment39 JSON");
    assert_eq!(
        live39.bodies.len(),
        parsed39.bodies.len(),
        "increment39 JSON must stay unchanged vs increment39_scene()"
    );
    assert_eq!(
        live39.bodies.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
        parsed39.bodies.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
        "increment39 JSON body ids must match increment39_scene()"
    );

    let live40 = increment40_scene();
    assert_eq!(
        live39.lights.len(),
        live40.lights.len(),
        "increment40 must not add lights[] entries vs increment 39"
    );
    assert_eq!(
        live39.bodies.len() + 1,
        live40.bodies.len(),
        "increment40 must add only the rider body vs increment 39"
    );
    assert_eq!(
        live39.joints.len(),
        live40.joints.len(),
        "increment40 must not add joints vs increment 39"
    );
    assert_eq!(
        live39.impulses.len(),
        live40.impulses.len(),
        "increment40 must keep the ball impulse vs increment 39"
    );
    assert!(live40.bodies.iter().any(|b| b.id == "rider"));
    assert!(live40.bodies.iter().any(|b| b.id == "platform"));
    assert_eq!(
        live40.bodies.iter().filter(|b| b.kinematic).count(),
        1,
        "increment40 must keep exactly one kinematic body (the platform)"
    );
    assert!(!body_by_id(&live40, "rider").kinematic, "rider must not be kinematic");
    let _ = increment20_scene();
    let _ = increment21_scene();
    let _ = increment22_scene();
    let _ = increment23_scene();
    let _ = increment24_scene();
    let _ = increment25_scene();
    let _ = increment26_scene();
    let _ = increment27_scene();
    let _ = increment28_scene();
    let _ = increment29_scene();
    let _ = increment30_scene();
    let _ = increment31_scene();
    let _ = increment32_scene();
    let _ = increment33_scene();
    let _ = increment34_scene();
    let _ = increment35_scene();
    let _ = increment36_scene();
    let _ = increment37_scene();
    let _ = increment38_scene();
}

#[test]
fn increment40_authors_the_rider() {
    let parsed = parse_scene(increment40_scene_json()).expect("increment40 JSON should parse");
    let parsed_rider = parsed
        .bodies
        .iter()
        .find(|b| b.id == "rider")
        .expect("increment40 JSON must author rider");
    assert_rider(parsed_rider);
    assert_platform(
        parsed
            .bodies
            .iter()
            .find(|b| b.id == "platform")
            .expect("increment40 JSON must keep platform"),
    );
    assert_eq!(
        parsed.camera.position,
        increment39_scene().camera.position,
        "parsed camera must stay increment-39"
    );
    assert_eq!(parsed.camera.look_at, increment39_scene().camera.look_at);
    assert_ball_impulse(&parsed.impulses);

    let live = increment40_scene();
    let live_rider = live
        .bodies
        .iter()
        .find(|b| b.id == "rider")
        .expect("increment40_scene() must author rider");
    assert_rider(live_rider);
    assert_platform(body_by_id(&live, "platform"));
    assert_eq!(live.camera.position, [3.6, 2.35, 5.2]);
    assert_eq!(live.camera.look_at, [0.1, 0.38, 0.0]);
    assert_ball_impulse(&live.impulses);

    let prior = increment39_scene();
    assert_no_rider(&prior, "increment39_scene()");
    assert_platform(body_by_id(&prior, "platform"));
}

#[test]
fn increment40_keeps_courtyard() {
    let scene = increment40_scene();
    assert!(
        scene.bodies.len() >= 13,
        "scene must have courtyard + lantern + drawer + charm + lid + platform + rider, got {}",
        scene.bodies.len()
    );
    assert_eq!(
        scene.triggers.len(),
        1,
        "keep the increment-32 drawer_open trigger, got {}",
        scene.triggers.len()
    );
    assert_eq!(scene.triggers[0].id, "drawer_open");
    match scene.triggers[0].shape {
        Shape::Box { size } => {
            assert!(
                (size[0] - 0.30).abs() < 1e-5
                    && (size[1] - 0.22).abs() < 1e-5
                    && (size[2] - 0.28).abs() < 1e-5,
                "trigger box size should stay [0.30, 0.22, 0.28], got {size:?}"
            );
        }
        _ => panic!("trigger should stay a box, got {:?}", scene.triggers[0].shape),
    }
    assert_eq!(scene.triggers[0].position, [-0.35, 0.10, 1.37]);

    let inc39 = increment39_scene();
    assert_eq!(
        scene.lights.len(),
        inc39.lights.len(),
        "no extra lights[] entries vs increment 39, got {} vs {}",
        scene.lights.len(),
        inc39.lights.len()
    );
    assert_eq!(
        scene.bodies.len(),
        inc39.bodies.len() + 1,
        "only the rider body vs increment 39"
    );
    assert_eq!(
        scene.joints.len(),
        inc39.joints.len(),
        "no new joints vs increment 39"
    );
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    assert_ball_impulse(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));

    let lantern = lantern_of(&scene);
    let e = lantern.material.emissive;
    assert!(
        (e[0] - 1.0).abs() < 1e-5 && (e[1] - 0.55).abs() < 1e-5 && (e[2] - 0.12).abs() < 1e-5,
        "keep lantern emissive [1.0, 0.55, 0.12], got {e:?}"
    );
    assert!(
        (lantern.material.emissive_intensity - 16.0).abs() < 1e-5,
        "keep lantern emissive_intensity 16, got {}",
        lantern.material.emissive_intensity
    );

    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "keep the authored area light size, got {size:?}"
    );
    assert!(intensity > 1.0, "area light intensity {intensity}");

    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());
    assert!(
        !scene.lights.iter().any(|l| matches!(l, Light::Point { .. })),
        "mesh light must not be a lights[] Point entry"
    );

    let has_bowl = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bowl")));
    let has_rock = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("rock")));
    let has_ball = scene
        .bodies
        .iter()
        .any(|b| b.id == "ball" && matches!(b.shape, Shape::Sphere { .. }));
    let has_pillar = body_by_id(&scene, "pillar").id == "pillar";
    let has_pane = matches!(
        &body_by_id(&scene, "pane").shape,
        Shape::Mesh { path, .. } if path.contains("pane")
    );
    let has_crate = matches!(
        &body_by_id(&scene, "crate").shape,
        Shape::Mesh { path, .. } if path.contains("crate")
    );
    let has_bench = matches!(
        &body_by_id(&scene, "bench").shape,
        Shape::Mesh { path, .. } if path.contains("bench")
    );
    let has_lantern = body_by_id(&scene, "lantern").id == "lantern";
    let has_drawer = body_by_id(&scene, "drawer").id == "drawer";
    let has_charm = body_by_id(&scene, "charm").id == "charm";
    let has_lid = body_by_id(&scene, "lid").id == "lid";
    let has_platform = body_by_id(&scene, "platform").id == "platform";
    let has_rider = body_by_id(&scene, "rider").id == "rider";
    assert!(
        has_bowl
            && has_rock
            && has_ball
            && has_pillar
            && has_pane
            && has_crate
            && has_bench
            && has_lantern
            && has_drawer
            && has_charm
            && has_lid
            && has_platform
            && has_rider,
        "keep the increment-39 courtyard including lid, charm, drawer, lantern, crate, bench, pane, platform, rider"
    );

    let hinges: Vec<_> = scene
        .joints
        .iter()
        .filter(|j| matches!(j, Joint::Hinge { .. }))
        .collect();
    assert_eq!(hinges.len(), 1, "keep the lantern hinge, got {}", hinges.len());
    match hinges[0] {
        Joint::Hinge { body_a, body_b, .. } => {
            assert_eq!(body_a, "pillar");
            assert_eq!(body_b, "lantern");
        }
        _ => unreachable!(),
    }
    let (v, f) = hinge_motor(&scene);
    assert!((v - 4.0).abs() < 1e-5, "keep hinge motor 4, got {v}");
    assert!((f - 8.0).abs() < 1e-5, "keep hinge motor force 8, got {f}");
    let (a, b, axis, limits, sv, sf) = slider_of(&scene);
    assert_eq!(a, "crate");
    assert_eq!(b, "drawer");
    let alen = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    assert!(alen > 0.5, "keep slider axis, got {axis:?}");
    assert!(limits[1] > limits[0], "keep slider limits");
    assert!((sv - (-2.0)).abs() < 1e-5, "slider motor_target_velocity -2, got {sv}");
    assert!((sf - 6.0).abs() < 1e-5, "slider motor_max_force 6, got {sf}");
    let (ba, bb, _anchor) = ball_of(&scene);
    assert_eq!(ba, "lantern");
    assert_eq!(bb, "charm");
    let (fa, fb, _fanchor) = fixed_of(&scene);
    assert_eq!(fa, "crate");
    assert_eq!(fb, "lid");

    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.clearcoat > 0.5,
        "keep the clearcoat ball, got {}",
        ball.material.clearcoat
    );
    assert!(
        (ball.material.clearcoat_roughness - 0.08).abs() < 1e-5,
        "keep clearcoat_roughness 0.08, got {}",
        ball.material.clearcoat_roughness
    );
    assert!(
        ball.material.anisotropy > 0.5,
        "keep the anisotropy ball, got {}",
        ball.material.anisotropy
    );
    assert!(
        (ball.material.anisotropy_rotation - 0.6).abs() < 1e-5,
        "keep anisotropy_rotation 0.6, got {}",
        ball.material.anisotropy_rotation
    );
    assert!(
        ball.material.iridescence > 0.5,
        "keep the iridescence ball, got {}",
        ball.material.iridescence
    );
    assert!(
        (ball.material.iridescence_ior - 1.3).abs() < 1e-5,
        "keep iridescence_ior 1.3, got {}",
        ball.material.iridescence_ior
    );
    assert!(
        (ball.material.iridescence_thickness - 380.0).abs() < 1e-3,
        "keep iridescence_thickness 380, got {}",
        ball.material.iridescence_thickness
    );
    assert_eq!(ball.position, [-1.10, 0.36, 0.10]);

    let bench = body_by_id(&scene, "bench");
    assert!(
        bench.material.sheen > 0.5,
        "keep the sheen bench, got {}",
        bench.material.sheen
    );
    assert!(
        (bench.material.sheen_roughness - 0.4).abs() < 1e-5,
        "keep sheen_roughness 0.4, got {}",
        bench.material.sheen_roughness
    );
    let c = bench.material.sheen_color;
    assert!(
        c[0] > 0.5 && c[1] < 0.35 && c[2] < 0.45,
        "keep bench sheen_color, got {c:?}"
    );

    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "keep pane dispersion, got {}",
        pane.material.dispersion
    );

    let cam = &scene.camera;
    assert_eq!(cam.position, [3.6, 2.35, 5.2]);
    assert_eq!(cam.look_at, [0.1, 0.38, 0.0]);

    let pillar_txt = fs::read_to_string(pillar_gltf_path()).unwrap();
    let pv: serde_json::Value = serde_json::from_str(&pillar_txt).unwrap();
    assert!(
        pv["materials"][0].get("occlusionTexture").is_some(),
        "must keep pillar occlusionTexture (AO)"
    );

    let pane_txt = fs::read_to_string(pane_gltf_path()).unwrap();
    let pane_v: serde_json::Value = serde_json::from_str(&pane_txt).unwrap();
    let (ior, transmission) = pane_ior_and_transmission(&pane_v["materials"][0]);
    assert!(transmission > 0.0, "keep pane transmission, got {transmission}");
    assert!(ior > 1.0, "keep pane IOR > 1, got {ior}");
    assert!((ior - 1.5).abs() < 0.05, "keep pane IOR ~1.5, got {ior}");
    let used = pane_v["extensionsUsed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>();
    assert!(
        used.iter().any(|s| *s == "KHR_materials_volume"),
        "keep pane volume attenuation"
    );
    assert!(
        used.iter().any(|s| *s == "KHR_materials_dispersion"),
        "keep pane dispersion"
    );
    let vol = &pane_v["materials"][0]["extensions"]["KHR_materials_volume"];
    let att = vol["attenuationColor"]
        .as_array()
        .expect("attenuationColor");
    let ac = [
        att[0].as_f64().unwrap(),
        att[1].as_f64().unwrap(),
        att[2].as_f64().unwrap(),
    ];
    assert!(
        (ac[0] - 0.20).abs() < 0.02 && (ac[1] - 0.85).abs() < 0.02 && (ac[2] - 0.28).abs() < 0.02,
        "keep volume attenuationColor [0.20, 0.85, 0.28], got {ac:?}"
    );
    let ad = vol["attenuationDistance"].as_f64().unwrap();
    assert!(
        (ad - 0.45).abs() < 0.02,
        "keep attenuationDistance 0.45, got {ad}"
    );
}

#[test]
fn increment40_rider_rides_and_dump_records() {
    let scene = increment40_scene();
    let dump = step_physics(&scene, INCREMENT40_STEPS, DEFAULT_DT).expect("physics");
    assert_ball_impulse(&dump.impulses);

    let authored_x = body_by_id(&scene, "platform").position[0];
    let platform = dump
        .bodies
        .iter()
        .find(|b| b.id == "platform")
        .expect("dump missing platform");
    assert!(
        platform.kinematic,
        "dump platform must record kinematic: true"
    );
    assert!(
        (platform.position[0] - authored_x).abs() > 0.4,
        "platform should slide +X by more than 0.4 (0.45*2s ≈ 0.9), authored_x={authored_x} got {:?}",
        platform.position
    );
    let vel = platform.linear_velocity;
    assert!(
        (vel[0] - 0.45).abs() < 1e-4 && vel[1].abs() < 1e-4 && vel[2].abs() < 1e-4,
        "dump platform linvel should stay [0.45, 0, 0], got {vel:?}"
    );

    let authored_rider_x = body_by_id(&scene, "rider").position[0];
    let rider = dump
        .bodies
        .iter()
        .find(|b| b.id == "rider")
        .expect("dump missing rider");
    assert!(
        !rider.kinematic,
        "dump rider must not be kinematic"
    );
    assert!(
        (rider.position[0] - authored_rider_x).abs() > 0.35,
        "rider should ride +X by more than 0.35, authored_x={authored_rider_x} got {:?}",
        rider.position
    );
    assert!(
        rider.position[1] > 0.10,
        "rider COM should stay on the slab (y > 0.10, not ground ~0.08), got {:?}",
        rider.position
    );

    let ball = dump
        .bodies
        .iter()
        .find(|b| b.id == "ball")
        .expect("dump missing ball");
    assert!(
        (ball.position[0] + 1.1).abs() > 0.25,
        "gold ball COM should roll off the increment-37 seat at x=-1.1, got {:?}",
        ball.position
    );

    assert!(
        dump.joints.len() >= 4,
        "dump must record hinge + slider + ball + fixed, got {}",
        dump.joints.len()
    );

    let hinge = dump
        .joints
        .iter()
        .find(|j| j.kind == "hinge")
        .expect("dump missing hinge");
    assert_eq!(hinge.body_a, "pillar");
    assert_eq!(hinge.body_b, "lantern");
    let mtv = hinge
        .motor_target_velocity
        .expect("dump hinge must record motor_target_velocity");
    let mmf = hinge
        .motor_max_force
        .expect("dump hinge must record motor_max_force");
    assert!(
        (mtv - 4.0).abs() < 1e-5,
        "dump hinge motor_target_velocity should be 4, got {mtv}"
    );
    assert!(
        (mmf - 8.0).abs() < 1e-5,
        "dump hinge motor_max_force should be 8, got {mmf}"
    );

    let slider = dump
        .joints
        .iter()
        .find(|j| j.kind == "slider")
        .expect("dump missing slider");
    assert_eq!(slider.body_a, "crate");
    assert_eq!(slider.body_b, "drawer");
    let slen = (slider.axis[0] * slider.axis[0]
        + slider.axis[1] * slider.axis[1]
        + slider.axis[2] * slider.axis[2])
        .sqrt();
    assert!(slen > 0.5, "dump slider axis must be present, got {:?}", slider.axis);
    let limits = slider.limits.expect("dump slider must record limits");
    assert!(
        limits[1] > limits[0],
        "dump slider limits must be a range, got {limits:?}"
    );
    let smtv = slider
        .motor_target_velocity
        .expect("dump slider must record motor_target_velocity");
    let smmf = slider
        .motor_max_force
        .expect("dump slider must record motor_max_force");
    assert!(
        (smtv - (-2.0)).abs() < 1e-5,
        "dump slider motor_target_velocity should be -2, got {smtv}"
    );
    assert!(
        (smmf - 6.0).abs() < 1e-5,
        "dump slider motor_max_force should be 6, got {smmf}"
    );

    let ballj = dump
        .joints
        .iter()
        .find(|j| j.kind == "ball")
        .expect("dump missing ball");
    assert_eq!(ballj.body_a, "lantern");
    assert_eq!(ballj.body_b, "charm");
    assert!(
        ballj.anchor.iter().any(|c| c.abs() > 1e-4),
        "dump ball anchor must be present, got {:?}",
        ballj.anchor
    );

    let fixed = dump
        .joints
        .iter()
        .find(|j| j.kind == "fixed")
        .expect("dump missing fixed");
    assert_eq!(fixed.body_a, "crate");
    assert_eq!(fixed.body_b, "lid");
    assert!(
        (fixed.anchor[0] + 0.35).abs() < 0.05
            && (fixed.anchor[1] - 0.26).abs() < 0.05
            && (fixed.anchor[2] - 0.85).abs() < 0.05,
        "dump fixed anchor should be the crate–lid interface, got {:?}",
        fixed.anchor
    );

    let lid = dump
        .bodies
        .iter()
        .find(|b| b.id == "lid")
        .expect("dump missing lid");
    assert!(
        lid.position[1] > 0.20,
        "lid COM should stay on the crate (y near authored 0.28, not on the ground), got {:?}",
        lid.position
    );

    let pillar = dump
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("dump missing pillar");
    assert_eq!(pillar.collider, "convex_hull", "pillar stays convex_hull");

    let grounded = dump.contacts.iter().any(|c| {
        (c.body_a == "pillar" || c.body_b == "pillar")
            && (c.body_a == "ground" || c.body_b == "ground")
    });
    assert!(
        grounded,
        "expected a ground–pillar contact, contacts={:?}",
        dump.contacts
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
    assert_eq!(hit.body, "drawer", "sweep should hit the drawer, got {}", hit.body);
    assert!(
        hit.toi > 0.0 && hit.toi < max_toi,
        "sweep toi should be in (0, max_toi={max_toi}), got {}",
        hit.toi
    );
    assert!(
        hit.point.iter().any(|c| c.abs() > 1e-4),
        "sweep point must be present, got {:?}",
        hit.point
    );
    let ray_hit = dump
        .ray_hits
        .iter()
        .find(|h| h.ray == "drawer_probe")
        .expect("dump ray_hits must still record drawer_probe");
    assert_eq!(
        ray_hit.body, "drawer",
        "probe should still hit the drawer, got {}",
        ray_hit.body
    );
    let _ = &dump.overlaps;
}

#[test]
fn increment40_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment40.sh");
    assert!(script.is_file(), "scripts/increment40.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment40-threejs.sh");
    assert!(three.is_file(), "scripts/increment40-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment40-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment40(&out, INCREMENT40_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment40");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert_ball_impulse(&scene.impulses);
    assert_platform(body_by_id(&scene, "platform"));
    assert_rider(body_by_id(&scene, "rider"));
    assert!(scene.bodies.iter().any(|b| b.id == "lid"));
    let (fa, fb, fanchor) = fixed_of(&scene);
    assert_eq!(fa, "crate");
    assert_eq!(fb, "lid");
    assert!(fanchor.iter().any(|c| c.abs() > 1e-4));
    assert_eq!(scene.shapecasts.len(), 1);
    assert_eq!(scene.shapecasts[0].id, "drawer_sweep");
    assert_eq!(scene.raycasts.len(), 1);
    assert_eq!(scene.raycasts[0].id, "drawer_probe");
    let lantern = lantern_of(&scene);
    let e = lantern.material.emissive;
    assert!(
        (e[0] - 1.0).abs() < 1e-5 && (e[1] - 0.55).abs() < 1e-5 && (e[2] - 0.12).abs() < 1e-5,
        "written lantern emissive {e:?}"
    );
    assert!(
        (lantern.material.emissive_intensity - 16.0).abs() < 1e-5,
        "written lantern emissive_intensity {}",
        lantern.material.emissive_intensity
    );
    assert_eq!(scene.triggers.len(), 1);
    assert_eq!(scene.triggers[0].id, "drawer_open");
    match scene.triggers[0].shape {
        Shape::Box { size } => {
            assert!((size[0] - 0.30).abs() < 1e-5);
            assert!((size[1] - 0.22).abs() < 1e-5);
            assert!((size[2] - 0.28).abs() < 1e-5);
        }
        _ => panic!("written trigger must be a box"),
    }
    assert_eq!(scene.triggers[0].position, [-0.35, 0.10, 1.37]);
    assert!(scene.bodies.iter().any(|b| b.id == "charm"));
    assert!(scene.bodies.iter().any(|b| b.id == "drawer"));
    assert!(scene.bodies.iter().any(|b| b.id == "lantern"));
    let (a, b, anchor) = ball_of(&scene);
    assert_eq!(a, "lantern");
    assert_eq!(b, "charm");
    assert!(anchor.iter().any(|c| c.abs() > 1e-4));
    let (v, f) = hinge_motor(&scene);
    assert!((v - 4.0).abs() < 1e-5, "written scene motor_target_velocity {v}");
    assert!((f - 8.0).abs() < 1e-5, "written scene motor_max_force {f}");
    let (sa, sb, _axis, limits, sv, sf) = slider_of(&scene);
    assert_eq!(sa, "crate");
    assert_eq!(sb, "drawer");
    assert!(limits[1] > limits[0]);
    assert!((sv - (-2.0)).abs() < 1e-5, "written slider motor_target_velocity {sv}");
    assert!((sf - 6.0).abs() < 1e-5, "written slider motor_max_force {sf}");
    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "written scene must keep dispersion on the pane, got {}",
        pane.material.dispersion
    );
    let ball = body_by_id(&scene, "ball");
    assert!(ball.material.iridescence > 0.5);
    assert!(ball.material.anisotropy > 0.5);
    assert!(ball.material.clearcoat > 0.5);
    let bench = body_by_id(&scene, "bench");
    assert!(bench.material.sheen > 0.5);
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(size[0] > 0.5 && size[1] > 0.4);
    assert!(intensity > 0.0);
    assert_eq!(scene.lights.len(), 2, "written scene must not grow lights[]");
    assert!(
        !scene.lights.iter().any(|l| matches!(l, Light::Point { .. })),
        "written scene must not add a Point light for the lantern"
    );
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pane"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pillar"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("crate"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bench"))));

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 13);
    let impulses = v["impulses"].as_array().expect("dump must have impulses array");
    assert_eq!(impulses.len(), 1);
    assert_eq!(impulses[0]["body"], "ball");
    let lin = impulses[0]["linear"].as_array().expect("impulse linear");
    assert!((lin[0].as_f64().unwrap() - 1.8).abs() < 1e-5);
    assert!((lin[1].as_f64().unwrap() - 0.4).abs() < 1e-5);
    assert!((lin[2].as_f64().unwrap() - 0.5).abs() < 1e-5);
    let ball_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "ball")
        .expect("dump should record the ball");
    let bx = ball_state["position"][0].as_f64().unwrap();
    assert!(
        (bx + 1.1).abs() > 0.25,
        "written dump ball x should have left the seat at -1.1, got {bx}"
    );
    let platform_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "platform")
        .expect("dump should record the platform");
    assert_eq!(
        platform_state["kinematic"].as_bool().unwrap_or(false),
        true,
        "written dump platform must record kinematic: true"
    );
    let px = platform_state["position"][0].as_f64().unwrap();
    let authored_x = body_by_id(&scene, "platform").position[0] as f64;
    assert!(
        (px - authored_x).abs() > 0.4,
        "written dump platform x should have slid > 0.4 from {authored_x}, got {px}"
    );
    let rider_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "rider")
        .expect("dump should record the rider");
    assert_eq!(
        rider_state["kinematic"].as_bool().unwrap_or(false),
        false,
        "written dump rider must not be kinematic"
    );
    let rx = rider_state["position"][0].as_f64().unwrap();
    let authored_rx = body_by_id(&scene, "rider").position[0] as f64;
    assert!(
        (rx - authored_rx).abs() > 0.35,
        "written dump rider x should have ridden > 0.35 from {authored_rx}, got {rx}"
    );
    let ry = rider_state["position"][1].as_f64().unwrap();
    assert!(
        ry > 0.10,
        "written dump rider y should stay on the slab (> 0.10), got {ry}"
    );
    let joints = v["joints"].as_array().expect("dump must have joints array");
    let fixed = joints
        .iter()
        .find(|j| j["kind"] == "fixed")
        .expect("dump joints must record the fixed weld");
    assert_eq!(fixed["body_a"], "crate");
    assert_eq!(fixed["body_b"], "lid");
    assert!(fixed["anchor"].is_array());
    let ballj = joints
        .iter()
        .find(|j| j["kind"] == "ball")
        .expect("dump joints must record the ball");
    assert_eq!(ballj["body_a"], "lantern");
    assert_eq!(ballj["body_b"], "charm");
    assert!(ballj["anchor"].is_array());
    let slider = joints
        .iter()
        .find(|j| j["kind"] == "slider")
        .expect("dump joints must record the slider");
    assert_eq!(slider["body_a"], "crate");
    assert_eq!(slider["body_b"], "drawer");
    assert!(slider["axis"].is_array());
    assert!(slider["limits"].is_array());
    assert!((slider["motor_target_velocity"].as_f64().unwrap() + 2.0).abs() < 1e-5);
    assert!((slider["motor_max_force"].as_f64().unwrap() - 6.0).abs() < 1e-5);
    let hinge = joints
        .iter()
        .find(|j| j["kind"] == "hinge")
        .expect("dump joints must record the hinge");
    assert_eq!(hinge["body_a"], "pillar");
    assert_eq!(hinge["body_b"], "lantern");
    assert!((hinge["motor_target_velocity"].as_f64().unwrap() - 4.0).abs() < 1e-5);
    assert!((hinge["motor_max_force"].as_f64().unwrap() - 8.0).abs() < 1e-5);
    let lid_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "lid")
        .expect("dump should record the lid");
    let ly = lid_state["position"][1].as_f64().unwrap();
    assert!(
        ly > 0.20,
        "written dump lid y should stay on the crate (> 0.20), got {ly}"
    );
    let sweeps = v["sweep_hits"].as_array().expect("dump must have sweep_hits");
    let hit = sweeps
        .iter()
        .find(|h| h["sweep"] == "drawer_sweep")
        .expect("dump sweep_hits must record drawer_sweep");
    assert_eq!(hit["body"], "drawer");
    let toi = hit["toi"].as_f64().unwrap();
    assert!(toi > 0.0 && toi < 1.0, "written sweep toi should be < max_toi, got {toi}");
    assert!(hit["point"].is_array());
    assert!(hit["normal"].is_array());
    let rays = v["ray_hits"].as_array().expect("dump must still have ray_hits");
    let ray_hit = rays
        .iter()
        .find(|h| h["ray"] == "drawer_probe")
        .expect("dump ray_hits must still record drawer_probe");
    assert_eq!(ray_hit["body"], "drawer");
    let drawer_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "drawer")
        .expect("dump should record the drawer");
    let dz = drawer_state["position"][2].as_f64().unwrap();
    assert!(
        dz < 1.15,
        "written dump drawer z should be closed (< 1.15), got {dz}"
    );
    let pillar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "pillar")
        .expect("dump should record the pillar");
    assert_eq!(
        pillar_state["collider"].as_str().unwrap_or(""),
        "convex_hull"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let grounded = contacts.iter().any(|c| {
        let a = c["body_a"].as_str().unwrap_or("");
        let b = c["body_b"].as_str().unwrap_or("");
        (a == "pillar" || b == "pillar") && (a == "ground" || b == "ground")
    });
    assert!(grounded, "dump contacts should include ground–pillar");

    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
