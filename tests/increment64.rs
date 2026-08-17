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
    increment57_scene_json, increment58_scene_json, increment59_scene_json,
    increment60_scene_json, increment61_scene_json, increment62_scene_json,
    increment63_scene, increment63_scene_json, increment64_scene, increment64_scene_json,
    parse_scene, run_increment64, step_catalog_scene_with_carry,
    step_physics, apply_win, DEFAULT_DT, INCREMENT63_STEPS, INCREMENT64_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 46] {
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
        ("61", increment61_scene_json()),
        ("62", increment62_scene_json()),
        ("63", increment63_scene_json()),
    ]
}

fn has_win_key(json: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(json).expect("parse scene json");
    v.get("win").is_some()
}

fn drops_len(json: &str) -> usize {
    let v: serde_json::Value = serde_json::from_str(json).expect("parse scene json");
    v.get("drops")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

#[test]
fn increment64_does_not_mutate_prior() {
    assert!(
        increment63_scene().drops.is_empty(),
        "increment63_scene must stay no-drop"
    );
    let win = increment63_scene().win.expect("increment63 still has win");
    assert_eq!(win.kind, "delivered");
    assert_eq!(win.body, "token");
    let tr = increment63_scene()
        .transition
        .expect("increment63 still has transition");
    assert_eq!(tr.to, "courtyard");
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        if name == "63" {
            assert!(parsed.drops.is_empty(), "increment63 still no-drop");
            assert!(parsed.win.is_some(), "increment63 still has win");
            assert_eq!(drops_len(json), 0, "increment63 json still no drops");
            assert!(has_win_key(json), "increment63 json HAS win");
        } else if name == "62" {
            assert!(parsed.win.is_none(), "increment62 scene.win must stay none");
            assert!(parsed.drops.is_empty(), "increment62 still no drops");
        }
    }
    let inc64 = increment64_scene_json();
    assert!(has_win_key(inc64), "increment64 json HAS win");
    assert_eq!(drops_len(inc64), 1, "increment64 json HAS the drop");
    let v: serde_json::Value = serde_json::from_str(inc64).expect("inc64 json");
    assert_eq!(v["win"]["kind"], "delivered");
    assert_eq!(v["win"]["body"], "token");
    assert_eq!(v["drops"][0]["body"], "token");
    assert_eq!(v["drops"][0]["trigger"], "exit");
    assert_eq!(v["drops"][0]["by"], "walker");
}

#[test]
fn increment64_scene_is_win_plus_drop() {
    let scene = increment64_scene();
    assert_eq!(scene.id, "lane");
    let win = scene.win.as_ref().expect("win");
    assert_eq!(win.kind, "delivered");
    assert_eq!(win.body, "token");
    assert_eq!(scene.drops.len(), 1, "exactly the increment61 drop");
    assert_eq!(scene.drops[0].body, "token");
    assert_eq!(scene.drops[0].trigger, "exit");
    assert_eq!(scene.drops[0].by, "walker");
    assert_eq!(scene.drops[0].drop_offset, [0.22, -0.06, 0.0]);
    let tr = scene.transition.as_ref().expect("transition");
    assert_eq!(tr.to, "courtyard");
    assert!(scene.pickups.iter().any(|p| p.hold), "has hold pickup");
    assert!(scene.bodies.iter().any(|b| b.id == "npc"), "has npc");
}

#[test]
fn increment63_next_still_won() {
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts/increment63/next-physics.json");
    if art.is_file() {
        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
        assert_eq!(next["scene"], "courtyard");
        assert_eq!(next["steps"], 1);
        let won = next.get("won").expect("increment63 next-physics still won");
        assert_eq!(won["kind"], "delivered");
        assert_eq!(won["body"], "token");
        assert_eq!(won["scene"], "courtyard");
        assert!(
            next.get("lost").is_none(),
            "increment63 next-physics must have no lost key"
        );
        let bodies = next["bodies"].as_array().expect("next bodies");
        assert!(bodies.iter().any(|b| b["id"] == "token"), "HAS token");
        let walker = bodies.iter().find(|b| b["id"] == "walker").expect("walker");
        let token = bodies.iter().find(|b| b["id"] == "token").expect("token");
        let wp = walker["position"].as_array().expect("walker pos");
        let tp = token["position"].as_array().expect("token pos");
        let expect = [
            wp[0].as_f64().unwrap() + 0.16,
            wp[1].as_f64().unwrap() + 0.22,
            wp[2].as_f64().unwrap() + 0.00,
        ];
        for i in 0..3 {
            let got = tp[i].as_f64().unwrap();
            assert!(
                (got - expect[i]).abs() < 0.08,
                "increment63 token[{i}] got {got} want {}",
                expect[i]
            );
        }
    }
    let lane = step_physics(&increment63_scene(), INCREMENT63_STEPS, DEFAULT_DT)
        .expect("increment63 lane");
    let mut next = step_catalog_scene_with_carry(
        "courtyard",
        INCREMENT63_STEPS,
        DEFAULT_DT,
        Some(&lane),
    )
    .expect("carry increment63 into courtyard");
    apply_win(&mut next, increment63_scene().win.as_ref());
    assert!(next.won.is_some(), "increment63 courtyard carry dump.won");
    assert!(next.lost.is_none(), "increment63 courtyard carry dump.lost is none");
    let v = serde_json::to_value(&next).expect("serialize increment63 next");
    assert!(v.get("won").is_some(), "increment63 next dump has won");
    assert!(v.get("lost").is_none(), "increment63 next dump omits lost");
}

#[test]
fn increment61_next_still_no_outcome() {
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts/increment61/next-physics.json");
    if art.is_file() {
        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
        assert!(
            next.get("won").is_none(),
            "increment61 next-physics must have no won key"
        );
        assert!(
            next.get("lost").is_none(),
            "increment61 next-physics must have no lost key"
        );
    }
}

#[test]
fn increment64_lane_dump() {
    let dump = step_physics(&increment64_scene(), INCREMENT64_STEPS, DEFAULT_DT)
        .expect("increment64 physics");
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
    let held = dump
        .held
        .iter()
        .find(|h| h.id == "token")
        .expect("dump.held token");
    assert_eq!(held.by, "walker");
    assert_eq!(held.at_step, 66);
    let dropped = dump
        .dropped
        .iter()
        .find(|d| d.id == "token")
        .expect("dump.dropped token");
    assert_eq!(dropped.by, "walker");
    assert!(
        (90..=110).contains(&dropped.at_step),
        "dropped token ~99, got {}",
        dropped.at_step
    );
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("dump walker");
    let token = dump
        .bodies
        .iter()
        .find(|b| b.id == "token")
        .expect("dump token");
    let expect = [
        walker.position[0] + 0.22,
        walker.position[1] - 0.06,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - expect[i]).abs() < 0.04,
            "token at drop_offset[{i}] got {} want {}",
            token.position[i],
            expect[i]
        );
    }
    assert!(dump.won.is_none(), "lane dump.won omitted");
    assert!(dump.lost.is_none(), "lane dump.lost omitted");
    let v = serde_json::to_value(&dump).expect("serialize dump");
    assert!(v.get("won").is_none(), "serialized lane dump must omit won");
    assert!(v.get("lost").is_none(), "serialized lane dump must omit lost");
}

#[test]
fn increment64_next_lost() {
    let out = PathBuf::from("target/test-increment64-next");
    let _ = fs::remove_dir_all(&out);
    let _paths = run_increment64(&out, INCREMENT64_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment64");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let next_steps = next["steps"].as_u64().expect("next steps");
    assert!(
        (1..=31).contains(&next_steps),
        "next dump.steps 1..=31, got {next_steps}"
    );
    assert!(
        next_steps >= 25,
        "expect ~31 like increment61 (no carry inject), got {next_steps}"
    );
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().all(|b| b["id"] != "token"), "NO token");
    assert!(bodies.iter().any(|b| b["id"] == "bar"), "HAS bar");
    assert!(bodies.iter().all(|b| b["id"] != "npc"), "NO npc");
    assert!(next.get("held").is_none(), "held omitted");
    assert!(next.get("won").is_none(), "won omitted");
    let lost = next.get("lost").expect("dump.lost");
    assert_eq!(lost["kind"], "empty_handed");
    assert_eq!(lost["body"], "token");
    assert_eq!(lost["scene"], "courtyard");
    let walker = bodies.iter().find(|b| b["id"] == "walker").expect("walker");
    let wp = walker["position"].as_array().expect("walker pos");
    let expect = [0.8646_f64, 0.1837, 1.4494];
    for i in 0..3 {
        let got = wp[i].as_f64().unwrap();
        assert!(
            (got - expect[i]).abs() < 0.08,
            "walker[{i}] got {got} want {}",
            expect[i]
        );
    }
}

#[test]
fn increment64_writes() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment64.sh");
    assert!(script.is_file(), "scripts/increment64.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment64-threejs.sh");
    assert!(three.is_file(), "scripts/increment64-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment64-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment64(&out, INCREMENT64_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment64");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(has_win_key(&scene_txt), "scene.json has win");
    assert_eq!(drops_len(&scene_txt), 1, "scene.json has the drop");
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    assert!(v.get("won").is_none(), "physics.json may omit won");
    assert!(v.get("lost").is_none(), "physics.json may omit lost");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    assert!(next.get("won").is_none(), "next-physics.json omits won");
    let lost = next.get("lost").expect("next-physics.json has lost");
    assert_eq!(lost["kind"], "empty_handed");
    assert_eq!(lost["body"], "token");
    assert_eq!(lost["scene"], "courtyard");
    let next_frame = out.join("next-frame.png");
    assert!(next_frame.is_file(), "next-frame.png must exist");
    assert!(fs::metadata(&next_frame).unwrap().len() > 256);
}
