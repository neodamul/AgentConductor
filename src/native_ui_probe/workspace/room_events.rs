use super::{Probe, room};

pub(super) fn apply(probe: &mut Probe, event: room::Event) {
    match event {
        room::Event::Refresh => probe.refresh_room(),
        room::Event::Stop(reason) => probe.stop_room(reason),
        room::Event::Evidence => probe.load_evidence(),
        room::Event::Decision(decision, reason) => probe.decide_room(decision, reason),
        room::Event::Resume => probe.resume_room(),
        room::Event::Recover => probe.recover_room(),
        room::Event::ToolDecision(action, id) => probe.decide_tool(action, id),
        room::Event::Briefing => probe.load_briefing(),
        room::Event::ReturnToWorkspace => {
            probe.room_view = None;
            probe.workspace_step = 0;
            probe.refresh_workspace();
        }
        room::Event::TaskDetails(index) => probe.load_task_details(index),
        room::Event::TaskAction(action) => probe.send_task_action(action),
    }
}
