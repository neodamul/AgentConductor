use super::{Editor, Value};

impl Editor {
    pub fn is_registering(&self) -> bool {
        self.is_editing() && self.new_profile
    }

    pub fn open_new(&mut self, profile: &Value) {
        self.open(profile);
        self.new_profile = true;
    }

    pub fn proposed_profiles(&mut self, profiles: &[Value]) -> Result<Vec<Value>, String> {
        let mut proposed = profiles.to_vec();
        if !self.is_editing() {
            return Ok(proposed);
        }
        if !self.agent_id_is_unique(profiles) {
            return Err("Agent ID is already registered".into());
        }
        if self.new_profile && profiles.len() >= 16 {
            return Err("At most 16 agents can be registered".into());
        }
        let index = if self.new_profile {
            proposed.push(self.base_profile.clone());
            proposed.len() - 1
        } else {
            proposed
                .iter()
                .position(|profile| profile["id"] == self.origin["id"])
                .ok_or_else(|| "The edited agent is no longer registered".to_owned())?
        };
        self.apply(&mut proposed[index])?;
        self.applied = false;
        if self.new_profile {
            self.clear_renames();
        }
        Ok(proposed)
    }
}
