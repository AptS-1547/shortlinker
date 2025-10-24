//! Navigation and selection logic

use super::state::App;

impl App {
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.links.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        if !self.links.is_empty() {
            self.selected_index = self.links.len() - 1;
        }
    }

    pub fn page_up(&mut self) {
        if self.selected_index >= 10 {
            self.selected_index -= 10;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn page_down(&mut self) {
        let max_index = self.links.len().saturating_sub(1);
        if self.selected_index + 10 <= max_index {
            self.selected_index += 10;
        } else {
            self.selected_index = max_index;
        }
    }

    /// Filter links based on search query
    pub fn filter_links(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_links.clear();
            self.is_searching = false;
            return;
        }

        self.is_searching = true;
        let query = self.search_input.to_lowercase();

        self.filtered_links = self
            .links
            .iter()
            .filter(|(code, link)| {
                code.to_lowercase().contains(&query) || link.target.to_lowercase().contains(&query)
            })
            .map(|(code, _)| code.clone())
            .collect();

        self.selected_index = 0;
    }
}
