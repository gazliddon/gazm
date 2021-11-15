#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<S: PartialEq + Clone + Copy> {
    state: S,
    last_state: Option<S>,
}

#[allow(dead_code)]
impl<S: PartialEq + Clone + Copy> State<S> {
    pub fn new(state: S) -> Self {
        Self {
            state,
            last_state: None,
        }
    }

    pub fn set(&mut self, new_state: S) {
        let old_state = self.state;
        self.state = new_state;
        self.last_state = Some(old_state);
    }

    pub fn has_changed(&self) -> bool {
        Some(self.state) == self.last_state
    }

    pub fn clear_change(&mut self) {
        self.last_state = Some(self.state)
    }

    pub fn get(&self) -> S {
        self.state
    }


    pub fn get_previous(&self) -> Option<S> {
        self.last_state
    }
}
