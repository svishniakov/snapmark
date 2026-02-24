#[derive(Clone, Debug)]
pub struct UndoHistory<T: Clone> {
    stack: Vec<T>,
    cursor: usize,
}

impl<T: Clone> UndoHistory<T> {
    pub fn new(initial: T) -> Self {
        Self {
            stack: vec![initial],
            cursor: 0,
        }
    }

    pub fn push_snapshot(&mut self, value: T) {
        if self.cursor + 1 < self.stack.len() {
            self.stack.truncate(self.cursor + 1);
        }
        self.stack.push(value);
        self.cursor = self.stack.len().saturating_sub(1);
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.stack.len()
    }

    pub fn undo(&mut self) -> Option<T> {
        if !self.can_undo() {
            return None;
        }
        self.cursor -= 1;
        Some(self.stack[self.cursor].clone())
    }

    pub fn redo(&mut self) -> Option<T> {
        if !self.can_redo() {
            return None;
        }
        self.cursor += 1;
        Some(self.stack[self.cursor].clone())
    }

    pub fn clear_with(&mut self, value: T) {
        self.stack.clear();
        self.stack.push(value);
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::UndoHistory;

    #[test]
    fn undo_redo_flow() {
        let mut history = UndoHistory::new(vec![1]);
        history.push_snapshot(vec![1, 2]);
        history.push_snapshot(vec![1, 2, 3]);

        assert_eq!(history.undo(), Some(vec![1, 2]));
        assert_eq!(history.undo(), Some(vec![1]));
        assert_eq!(history.undo(), None);

        assert_eq!(history.redo(), Some(vec![1, 2]));
        history.push_snapshot(vec![9]);
        assert_eq!(history.redo(), None);
    }
}
