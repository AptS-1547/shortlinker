//! Navigation and selection logic

use super::state::App;
use crate::interfaces::tui::constants::PAGE_SCROLL_STEP;

#[cfg(feature = "tui")]
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

impl App {
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.table_state.select(Some(self.selected_index));
    }

    pub fn move_selection_down(&mut self) {
        let display_len = self.get_display_links().len();
        if self.selected_index < display_len.saturating_sub(1) {
            self.selected_index += 1;
        }
        self.table_state.select(Some(self.selected_index));
    }

    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
        self.table_state.select(Some(self.selected_index));
    }

    pub fn jump_to_bottom(&mut self) {
        let display_len = self.get_display_links().len();
        if display_len > 0 {
            self.selected_index = display_len - 1;
        }
        self.table_state.select(Some(self.selected_index));
    }

    pub fn page_up(&mut self) {
        if self.selected_index >= PAGE_SCROLL_STEP {
            self.selected_index -= PAGE_SCROLL_STEP;
        } else {
            self.selected_index = 0;
        }
        self.table_state.select(Some(self.selected_index));
    }

    pub fn page_down(&mut self) {
        let display_len = self.get_display_links().len();
        let max_index = display_len.saturating_sub(1);
        if self.selected_index + PAGE_SCROLL_STEP <= max_index {
            self.selected_index += PAGE_SCROLL_STEP;
        } else {
            self.selected_index = max_index;
        }
        self.table_state.select(Some(self.selected_index));
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
        self.table_state.select(Some(0));
    }

    /// Filter links using fuzzy matching (nucleo-matcher)
    #[cfg(feature = "tui")]
    pub fn filter_links_fuzzy(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_links.clear();
            self.is_searching = false;
            return;
        }

        self.is_searching = true;

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(
            &self.search_input,
            CaseMatching::Ignore,
            Normalization::Smart,
        );

        let mut scored_links: Vec<(String, u32)> = self
            .links
            .iter()
            .filter_map(|(code, link)| {
                // Try matching against code
                let mut code_buf = Vec::new();
                let code_str = Utf32Str::new(code, &mut code_buf);
                let code_score = pattern.score(code_str, &mut matcher);

                // Try matching against target URL
                let mut url_buf = Vec::new();
                let url_str = Utf32Str::new(&link.target, &mut url_buf);
                let url_score = pattern.score(url_str, &mut matcher);

                // Take the best score
                match (code_score, url_score) {
                    (Some(cs), Some(us)) => Some((code.clone(), cs.max(us))),
                    (Some(cs), None) => Some((code.clone(), cs)),
                    (None, Some(us)) => Some((code.clone(), us)),
                    (None, None) => None,
                }
            })
            .collect();

        // Sort by score descending
        scored_links.sort_by(|a, b| b.1.cmp(&a.1));

        self.filtered_links = scored_links.into_iter().map(|(code, _)| code).collect();
        self.selected_index = 0;
    }

    /// Filter links using fuzzy matching (fallback for non-tui builds)
    #[cfg(not(feature = "tui"))]
    pub fn filter_links_fuzzy(&mut self) {
        self.filter_links();
    }
}
