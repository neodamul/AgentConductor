use super::{Probe, room};

pub(super) fn apply(probe: &mut Probe, event: room::Event) {
    match event {
        room::Event::Refresh => probe.refresh_room(),
        room::Event::Stop(reason) => probe.stop_room(reason),
        room::Event::Evidence => probe.load_evidence(),
        room::Event::ScopeFailure => probe.load_scope_failure(),
        room::Event::ScopeRecoveryPreview { task, paths, test } => {
            probe.preview_scope_recovery(task, paths, test)
        }
        room::Event::ScopeRecoveryApply {
            task,
            paths,
            test,
            digest,
        } => probe.apply_scope_recovery(task, paths, test, digest),
        room::Event::Decision(decision, reason) => probe.decide_room(decision, reason),
        room::Event::Resume => probe.resume_room(),
        room::Event::Recover => probe.recover_room(),
        room::Event::ToolDecision(action, id) => probe.decide_tool(action, id),
        room::Event::Briefing => probe.load_briefing(),
        room::Event::Coordinator => probe.load_coordinator_judgments(),
        room::Event::ReturnToWorkspace => {
            probe.room_view = None;
            if let Some(worker) = probe.worker.as_mut() {
                worker.set_room_view(false);
            }
            probe.refresh_workspace();
        }
        room::Event::TaskDetails(index) => probe.load_task_details(index),
        room::Event::TaskAction(action) => probe.send_task_action(action),
    }
}
