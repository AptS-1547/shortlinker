//! Navigation and selection logic

use super::state::App;
use crate::errors::ShortlinkerError;
use crate::interfaces::tui::constants::PAGE_SCROLL_STEP;

impl App {
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }

    pub fn move_selection_down(&mut self) {
        let display_len = self.display_count();
        if self.selected_index < display_len.saturating_sub(1) {
            self.selected_index += 1;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }

    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.table_state.select(Some(self.selected_index));
    }

    pub fn jump_to_bottom(&mut self) {
        let display_len = self.display_count();
        if display_len > 0 {
            self.selected_index = display_len - 1;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }

    pub fn page_up(&mut self) {
        if self.selected_index >= PAGE_SCROLL_STEP {
            self.selected_index -= PAGE_SCROLL_STEP;
        } else {
            self.selected_index = 0;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }

    pub fn page_down(&mut self) {
        let display_len = self.display_count();
        let max_index = display_len.saturating_sub(1);
        if self.selected_index + PAGE_SCROLL_STEP <= max_index {
            self.selected_index += PAGE_SCROLL_STEP;
        } else {
            self.selected_index = max_index;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }

    /// 加载下一页
    pub async fn next_page(&mut self) -> Result<(), ShortlinkerError> {
        if self.has_next_page() {
            self.load_page(self.current_page + 1).await?;
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.table_state.select(Some(0));
        }
        Ok(())
    }

    /// 加载上一页
    pub async fn prev_page(&mut self) -> Result<(), ShortlinkerError> {
        if self.has_prev_page() {
            self.load_page(self.current_page - 1).await?;
            let last = self.display_count().saturating_sub(1);
            self.selected_index = last;
            self.adjust_scroll_offset();
            self.table_state.select(Some(self.selected_index));
        }
        Ok(())
    }

    /// 跳到第一页
    pub async fn first_page(&mut self) -> Result<(), ShortlinkerError> {
        if self.current_page != 1 {
            self.load_page(1).await?;
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.table_state.select(Some(0));
        } else {
            self.jump_to_top();
        }
        Ok(())
    }

    /// 跳到最后一页
    pub async fn last_page(&mut self) -> Result<(), ShortlinkerError> {
        let last = self.total_pages();
        if self.current_page != last {
            self.load_page(last).await?;
            let last_idx = self.display_count().saturating_sub(1);
            self.selected_index = last_idx;
            self.adjust_scroll_offset();
            self.table_state.select(Some(self.selected_index));
        } else {
            self.jump_to_bottom();
        }
        Ok(())
    }

    /// 调整 scroll_offset 确保 selected_index 在可见窗口内
    pub fn adjust_scroll_offset(&mut self) {
        let vh = self.last_visible_height.max(1);
        // 如果光标在可见窗口上方，向上滚动
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        // 如果光标在可见窗口下方，向下滚动
        if self.selected_index >= self.scroll_offset + vh {
            self.scroll_offset = self.selected_index - vh + 1;
        }
    }
}
