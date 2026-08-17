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
    increment51_scene_json, increment52_scene_json, increment53_scene_json,
    increment54_scene_json, increment55_scene_json, increment56_scene_json,
    increment57_scene_json, increment58_scene_json, increment59_scene,
    increment59_scene_json, increment60_scene, increment60_scene_json, increment61_scene,
    increment61_scene_json, parse_scene, run_increment61, scene_by_id,
    step_physics, DEFAULT_DT, INCREMENT59_STEPS, INCREMENT61_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 43] {
    [
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
        ("53", increment53_scene_json()),
        ("54", increment54_scene_json()),
        ("55", increment55_scene_json()),
        ("56", increment56_scene_json()),
        ("57", increment57_scene_json()),
        ("58", increment58_scene_json()),
        ("59", increment59_scene_json()),
        ("60", increment60_scene_json()),
    ]
}


#[test]
fn increment61_does_not_mutate_prior() {
    assert!(
        increment59_scene().transition.is_none(),
        "increment59_scene must stay transition-free"
    );
    assert_eq!(increment59_scene().id, "lane");
    let courtyard = increment60_scene();
    assert!(
        courtyard.bodies.iter().any(|b| b.id == "bar")
            || courtyard
                .play_until
                .as_ref()
                .map(|u| u.kind == "picked_up")
                .unwrap_or(false),
        "increment60 still courtyard catalog"
    );
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        assert!(
            parsed.transition.is_none(),
            "increment {name} must stay transition-free"
        );
        if name == "59" {
            assert!(
                !json.contains("\"transition\""),
                "increment59 JSON must still have no transition key"
            );
        }
    }
    let inc61 = increment61_scene_json();
    assert!(
        inc61.contains("\"transition\""),
        "increment61 json must HAVE transition"
    );
    assert!(
        inc61.contains("courtyard"),
        "increment61 json transition must be courtyard"
    );
}

#[test]
fn increment61_scene_is_lane_plus_transition() {
    let scene = increment61_scene();
    assert_eq!(scene.id, "lane");
    let tr = scene.transition.as_ref().expect("transition");
    assert_eq!(tr.to, "courtyard");
    assert!(scene.bodies.iter().any(|b| b.id == "npc"), "has npc");
    let until = scene.play_until.as_ref().expect("play_until");
    assert_eq!(until.kind, "entered");
    assert_eq!(until.body, "exit");
    assert!(!scene.drops.is_empty(), "has drops");
    assert!(scene.pickups.iter().any(|p| p.hold), "has hold");
}

#[test]
fn increment59_dump_still_no_transition() {
    let dump = step_physics(&increment59_scene(), INCREMENT59_STEPS, DEFAULT_DT)
        .expect("increment59 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "increment59 dump.steps ~100, got {}",
        dump.steps
    );
    assert!(dump.transition.is_none(), "increment59 dump has no transition");
    let json = serde_json::to_string(&dump).expect("serialize increment59 dump");
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse increment59 dump");
    assert!(
        v.get("transition").is_none(),
        "increment59 dump JSON must omit transition key"
    );
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "has npc");
}

#[test]
fn increment61_physics_lane_then_courtyard() {
    let dump = step_physics(&increment61_scene(), INCREMENT61_STEPS, DEFAULT_DT)
        .expect("increment61 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert!(
        (90..=110).contains(&stopped.at_step),
        "stopped entered/exit ~99, got {}",
        stopped.at_step
    );
    let tr = dump.transition.as_ref().expect("dump.transition");
    assert_eq!(tr.to, "courtyard");
    assert_eq!(tr.at_step, stopped.at_step);
    assert!(dump.bodies.iter().any(|b| b.id == "npc"), "HAS npc");
    assert!(dump.bodies.iter().any(|b| b.id == "walker"), "HAS walker");
    assert!(dump.bodies.iter().any(|b| b.id == "token"), "HAS token");
    assert!(!dump.dropped.is_empty(), "token dropped");
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("dump walker");
    let cam = dump.camera.as_ref().expect("dump.camera");
    let expect_pos = [
        walker.position[0] - 1.00,
        walker.position[1] + 0.80,
        walker.position[2] + 1.60,
    ];
    for i in 0..3 {
        assert!(
            (cam.position[i] - expect_pos[i]).abs() < 0.08,
            "follow-cam lane offset position[{i}] got {} want {}",
            cam.position[i],
            expect_pos[i]
        );
    }
}

#[test]
fn increment61_writes_next() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment61.sh");
    assert!(script.is_file(), "scripts/increment61.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment61-threejs.sh");
    assert!(three.is_file(), "scripts/increment61-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment61-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment61(&out, INCREMENT61_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment61");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(
        scene_txt.contains("\"transition\""),
        "scene.json has transition"
    );
    assert!(
        scene_txt.contains("courtyard"),
        "scene.json transition courtyard"
    );
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    assert_eq!(v["transition"]["to"], "courtyard");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let next_steps = next["steps"].as_u64().expect("next steps");
    assert!(
        (30..=31).contains(&next_steps),
        "next dump.steps 30..=31, got {next_steps}"
    );
    assert_eq!(next["stopped"]["kind"], "picked_up");
    assert_eq!(next["stopped"]["body"], "token");
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().any(|b| b["id"] == "bar"), "HAS bar");
    assert!(bodies.iter().all(|b| b["id"] != "npc"), "NO npc");
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let next_frame = out.join("next-frame.png");
    assert!(next_frame.is_file(), "next-frame.png must exist");
    assert!(fs::metadata(&next_frame).unwrap().len() > 256);
}

#[test]
fn increment60_still_works() {
    assert!(scene_by_id("courtyard").is_some());
}
