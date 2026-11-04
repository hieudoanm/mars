use crate::notebook::Notebook;

pub struct NotebookHistory {
    past: Vec<Notebook>,
    future: Vec<Notebook>,
    capacity: usize,
}

impl NotebookHistory {
    pub fn new(initial: Notebook, capacity: usize) -> Self {
        Self {
            past: vec![initial],
            future: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn current(&self) -> Option<&Notebook> {
        self.past.last()
    }

    pub fn record(&mut self, notebook: Notebook) {
        self.past.push(notebook);
        self.future.clear();
        if self.past.len() > self.capacity {
            self.past.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<&Notebook> {
        if self.past.len() <= 1 {
            return None;
        }
        let current = self.past.pop()?;
        self.future.push(current);
        self.past.last()
    }

    pub fn redo(&mut self) -> Option<&Notebook> {
        let next = self.future.pop()?;
        self.past.push(next);
        self.past.last()
    }
}
