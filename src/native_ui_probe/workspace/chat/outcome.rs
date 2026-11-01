use super::State;

impl State {
    pub fn outcome_unknown(&mut self, error: String) {
        if self.pending_turn.is_some() {
            self.requesting = false;
            self.uncertain_error = Some(error);
            self.error = None;
        } else {
            self.failed(error);
        }
    }
}
