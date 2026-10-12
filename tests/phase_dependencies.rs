//! DD-62: phases run in order, so a dependency on an earlier phase's task is satisfied.
use agentconductor::phases::{PhaseSpec, drop_satisfied_dependencies};

fn phase(id: &str, tasks: &[(&str, &[&str])]) -> PhaseSpec {
    PhaseSpec {
        id: id.into(),
        name: id.into(),
        tasks: tasks
            .iter()
            .map(|(task, deps)| {
                serde_json::from_value(serde_json::json!({
                    "id": task, "agent": "claude", "prompt": "p",
                    "paths": [format!("{task}.txt")], "depends_on": deps,
                    "test_command": ["true"],
                }))
                .unwrap()
            })
            .collect(),
    }
}

fn deps(phases: &[PhaseSpec]) -> Vec<Vec<String>> {
    phases
        .iter()
        .flat_map(|p| p.tasks.iter().map(|t| t.depends_on.clone()))
        .collect()
}

#[test]
fn earlier_phase_links_are_dropped_and_others_are_kept() {
    // Shape returned by Claude for a real Windows plan: each phase chained to the previous.
    let mut phases = vec![
        phase("01-requirements", &[("t01", &[]), ("t02", &["t01"])]),
        phase(
            "02-core",
            &[("t03", &["t02"]), ("t04", &["t03"]), ("t05", &["t99"])],
        ),
        phase("03-ui", &[("t06", &["t05", "t07"]), ("t07", &[])]),
    ];
    drop_satisfied_dependencies(&mut phases);
    let expected: Vec<Vec<String>> = vec![
        vec![],
        vec!["t01".into()],
        vec![],
        vec!["t03".into()],
        vec!["t99".into()],
        vec!["t07".into()],
        vec![],
    ];
    assert_eq!(deps(&phases), expected);
}

#[test]
fn later_phase_dependency_is_not_dropped() {
    let mut phases = vec![
        phase("01-a", &[("a", &["b"])]),
        phase("02-b", &[("b", &[])]),
    ];
    drop_satisfied_dependencies(&mut phases);
    assert_eq!(phases[0].tasks[0].depends_on, ["b"]);
}
