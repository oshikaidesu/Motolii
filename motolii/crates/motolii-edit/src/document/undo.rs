use super::Document;

impl Document {
    pub fn mark_undo_floor(&mut self) {
        self.floor = self.head;
    }

    pub fn can_undo(&self) -> bool {
        self.head > self.floor
    }

    /// 戻れる段数と進める段数。履歴を一覧にする側はこれだけ読む。
    pub fn history_depth(&self) -> (usize, usize) {
        (
            (self.head - self.floor).max(0) as usize,
            (self.tip - self.head).max(0) as usize,
        )
    }

    pub fn can_redo(&self) -> bool {
        self.head < self.tip
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.head -= 1;
            self.clear_all_transients();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.head += 1;
            self.clear_all_transients();
            true
        } else {
            false
        }
    }
}
