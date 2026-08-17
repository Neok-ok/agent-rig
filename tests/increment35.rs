use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment20_scene, increment20_scene_json, increment21_scene,
    increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene,
    increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene,
    increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene,
    increment27_scene_json, increment28_scene, increment28_scene_json, increment29_scene,
    increment29_scene_json, increment30_scene, increment30_scene_json, increment31_scene,
    increment31_scene_json, increment32_scene, increment32_scene_json, increment33_scene,
    increment33_scene_json, increment34_scene, increment34_scene_json, increment35_scene,
    increment35_scene_json, parse_scene, run_increment35, step_physics, Joint, Light, Shape,
    DEFAULT_DT, INCREMENT35_STEPS,
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

fn lantern_of(scene: &agent_rig::Scene) -> &agent_rig::Body {
    body_by_id(scene, "lantern")
}

fn json_has_raycasts(json: &str) -> bool {
    json.contains("\"raycasts\"") || json.contains("drawer_probe")
}

#[test]
fn increment35_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
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
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
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
    for (name, json) in [
        ("18", increment18_scene_json()),
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
    ] {
        assert!(
            !json_has_raycasts(json),
            "must not mutate increment {name} scene JSON with raycasts"
        );
    }

    assert!(
        s34.raycasts.is_empty(),
        "increment34 JSON must stay raycast-free, got {}",
        s34.raycasts.len()
    );
    let live34 = increment34_scene();
    assert!(
        live34.raycasts.is_empty(),
        "increment34_scene() must not grow raycasts, got {}",
        live34.raycasts.len()
    );
    let (_a, _b, _axis, _limits, v34, f34) = slider_of(&live34);
    assert!(
        (v34 - (-2.0)).abs() < 1e-5 && (f34 - 6.0).abs() < 1e-5,
        "increment34 slider motor must stay -2/6, got {v34}/{f34}"
    );
    let lantern34 = lantern_of(&live34);
    assert!(
        (lantern34.material.emissive_intensity - 16.0).abs() < 1e-5,
        "increment34 lantern must stay intensity 16, got {}",
        lantern34.material.emissive_intensity
    );
    assert_eq!(live34.triggers.len(), 1);
    assert_eq!(live34.triggers[0].id, "drawer_open");
    assert!(live34.bodies.iter().any(|b| b.id == "charm"));
    assert!(live34.joints.iter().any(|j| matches!(j, Joint::Ball { .. })));
    let (hv, hf) = hinge_motor(&live34);
    assert!(
        (hv - 4.0).abs() < 1e-5 && (hf - 8.0).abs() < 1e-5,
        "increment34 hinge motor must stay 4/8, got {hv}/{hf}"
    );
    assert_eq!(
        live34.lights.len(),
        increment35_scene().lights.len(),
        "increment35 must not add lights[] entries vs increment 34"
    );
    assert_eq!(
        live34.bodies.len(),
        increment35_scene().bodies.len(),
        "increment35 must not add bodies vs increment 34"
    );
    assert_eq!(
        live34.joints.len(),
        increment35_scene().joints.len(),
        "increment35 must not add joints vs increment 34"
    );
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
}

#[test]
fn increment35_authors_drawer_probe() {
    let parsed = parse_scene(increment35_scene_json()).expect("increment35 JSON should parse");
    assert_eq!(parsed.raycasts.len(), 1, "parsed scene should author one raycast");
    let ray = &parsed.raycasts[0];
    assert_eq!(ray.id, "drawer_probe");
    assert!(
        (ray.origin[0] + 0.35).abs() < 0.05
            && (ray.origin[1] - 0.55).abs() < 0.05
            && (ray.origin[2] - 1.35).abs() < 0.05,
        "parsed origin should be near [-0.35, 0.55, 1.35], got {:?}",
        ray.origin
    );
    let dlen = (ray.direction[0] * ray.direction[0]
        + ray.direction[1] * ray.direction[1]
        + ray.direction[2] * ray.direction[2])
        .sqrt();
    assert!((dlen - 1.0).abs() < 1e-3, "parsed direction should be unit, got {dlen}");
    let target = [-0.35_f32, 0.10, 1.02];
    let to_target = [
        target[0] - ray.origin[0],
        target[1] - ray.origin[1],
        target[2] - ray.origin[2],
    ];
    let tlen = (to_target[0] * to_target[0]
        + to_target[1] * to_target[1]
        + to_target[2] * to_target[2])
        .sqrt();
    let want = [to_target[0] / tlen, to_target[1] / tlen, to_target[2] / tlen];
    let dot = ray.direction[0] * want[0] + ray.direction[1] * want[1] + ray.direction[2] * want[2];
    assert!(
        dot > 0.99,
        "parsed direction should aim at seated drawer COM, dot={dot}"
    );
    assert!(
        (ray.max_toi - 2.0).abs() < 0.25,
        "parsed max_toi should be ~2, got {}",
        ray.max_toi
    );

    let live = increment35_scene();
    assert_eq!(live.raycasts.len(), 1, "live scene should author one raycast");
    let ray = &live.raycasts[0];
    assert_eq!(ray.id, "drawer_probe");
    assert!(
        (ray.origin[0] + 0.35).abs() < 0.05
            && (ray.origin[1] - 0.55).abs() < 0.05
            && (ray.origin[2] - 1.35).abs() < 0.05,
        "live origin should be near [-0.35, 0.55, 1.35], got {:?}",
        ray.origin
    );
    let dlen = (ray.direction[0] * ray.direction[0]
        + ray.direction[1] * ray.direction[1]
        + ray.direction[2] * ray.direction[2])
        .sqrt();
    assert!((dlen - 1.0).abs() < 1e-3, "live direction should be unit, got {dlen}");
    assert!(
        (ray.max_toi - 2.0).abs() < 0.25,
        "live max_toi should be ~2, got {}",
        ray.max_toi
    );

    let prior = increment34_scene();
    assert!(
        prior.raycasts.is_empty(),
        "increment34_scene() must stay raycast-free"
    );
}

#[test]
fn increment35_keeps_courtyard() {
    let scene = increment35_scene();
    assert!(
        scene.bodies.len() >= 10,
        "scene must have courtyard + lantern + drawer + charm, got {}",
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

    let inc34 = increment34_scene();
    assert_eq!(
        scene.lights.len(),
        inc34.lights.len(),
        "no extra lights[] entries vs increment 34, got {} vs {}",
        scene.lights.len(),
        inc34.lights.len()
    );
    assert_eq!(
        scene.bodies.len(),
        inc34.bodies.len(),
        "no extra bodies vs increment 34"
    );
    assert_eq!(
        scene.joints.len(),
        inc34.joints.len(),
        "no extra joints vs increment 34"
    );

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
            && has_charm,
        "keep the increment-34 courtyard including charm, drawer, lantern, crate, bench, pane"
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
fn increment35_dump_records_drawer_probe_hit() {
    let scene = increment35_scene();
    let dump = step_physics(&scene, INCREMENT35_STEPS, DEFAULT_DT).expect("physics");
    assert!(
        dump.joints.len() >= 3,
        "dump must record hinge + slider + ball, got {}",
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

    let max_toi = scene.raycasts[0].max_toi;
    let hit = dump
        .ray_hits
        .iter()
        .find(|h| h.ray == "drawer_probe")
        .expect("dump ray_hits must record drawer_probe");
    assert_eq!(hit.body, "drawer", "probe should hit the drawer, got {}", hit.body);
    assert!(
        hit.toi > 0.0 && hit.toi < max_toi,
        "hit toi should be in (0, max_toi={max_toi}), got {}",
        hit.toi
    );
    assert!(
        hit.point.iter().any(|c| c.abs() > 1e-4),
        "hit point must be present, got {:?}",
        hit.point
    );
    let _ = &dump.overlaps;
}

#[test]
fn increment35_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment35.sh");
    assert!(script.is_file(), "scripts/increment35.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment35-threejs.sh");
    assert!(three.is_file(), "scripts/increment35-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment35-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment35(&out, INCREMENT35_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment35");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
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
    assert!(v["bodies"].as_array().unwrap().len() >= 10);
    let joints = v["joints"].as_array().expect("dump must have joints array");
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
    let hits = v["ray_hits"].as_array().expect("dump must have ray_hits");
    let hit = hits
        .iter()
        .find(|h| h["ray"] == "drawer_probe")
        .expect("dump ray_hits must record drawer_probe");
    assert_eq!(hit["body"], "drawer");
    let toi = hit["toi"].as_f64().unwrap();
    assert!(toi > 0.0 && toi < 2.0, "written hit toi should be < max_toi, got {toi}");
    assert!(hit["point"].is_array());
    assert!(hit["normal"].is_array());
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
