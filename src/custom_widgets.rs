pub struct StatefulList<T> {
    pub state: usize,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>, initial_state: usize) -> StatefulList<T> {
        StatefulList { state: initial_state, items }
    }

    pub fn next(&mut self) {
        if self.state == self.items.len() - 1 {
            self.state = 0
        } else {
            self.state += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.state == 0 {
            self.state = self.items.len() - 1
        } else {
            self.state -= 1;
        }
    }
}
