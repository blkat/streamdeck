#[derive(Debug, Clone)]
pub struct Navigation {
    stack: Vec<i64>,
}

impl Navigation {
    pub fn new(root_id: i64) -> Self {
        Self {
            stack: vec![root_id],
        }
    }

    pub fn current(&self) -> i64 {
        *self.stack.last().expect("navigation stack empty")
    }

    pub fn push(&mut self, page_id: i64) {
        self.stack.push(page_id);
    }

    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn home(&mut self, root_id: i64) {
        self.stack.clear();
        self.stack.push(root_id);
    }

    pub fn is_at_root(&self) -> bool {
        self.stack.len() <= 1
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn stack(&self) -> &[i64] {
        &self.stack
    }
}

pub const HOME_ROW: i32 = 1;
pub const HOME_COL: i32 = 2;
pub const BACK_ROW: i32 = 2;
pub const BACK_COL: i32 = 4;

pub fn is_home_slot(row: i32, col: i32) -> bool {
    row == HOME_ROW && col == HOME_COL
}

pub fn is_back_slot(row: i32, col: i32) -> bool {
    row == BACK_ROW && col == BACK_COL
}

pub fn is_reserved_slot(row: i32, col: i32, at_root: bool) -> bool {
    if is_home_slot(row, col) {
        return true;
    }
    if !at_root && is_back_slot(row, col) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_stack() {
        let mut nav = Navigation::new(1);
        nav.push(2);
        assert_eq!(nav.current(), 2);
        nav.pop();
        assert_eq!(nav.current(), 1);
        nav.home(1);
        assert!(nav.is_at_root());
    }
}
