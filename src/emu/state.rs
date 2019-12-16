#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<S : PartialEq + Clone > {
    state : S,
    last_state : Option<S>,
}

impl<S : PartialEq + Clone > State <S> {
    pub fn new(state : &S) -> Self {
        Self {
            state : state.clone(),
            last_state : None,
        }
    }

    pub fn set(&mut self, new_state : &S) {
        self.last_state  = Some(self.state.clone());
        self.state = new_state.clone();
    }

    pub fn has_changed(&self) -> bool {
        if let Some(last_state) = self.last_state.clone() {
            last_state != self.state
        } else {
            false
        }
    }

    pub fn clear_change(&mut self) {
        self.last_state = Some(self.state.clone())
    }

    pub fn get(&self) -> S {
        self.state.clone()
    }
}

