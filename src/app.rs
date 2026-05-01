pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

pub struct Feature {
    pub name: String,
    pub enabled: bool,
    pub deps: String,
}

pub struct CrateDetail {
    #[allow(dead_code)]
    pub name: String,
    pub version: String,
    pub description: String,
    pub rust_version: Option<String>,
    pub repository: Option<String>,
    pub features: Vec<Feature>,
}

pub struct FeatureItem {
    pub name: String,
    pub selected: bool,
}

pub struct FeatureSelectState {
    pub crate_name: String,
    pub features: Vec<FeatureItem>,
    pub no_default_features: bool,
    pub cursor: usize,
}

impl FeatureSelectState {
    pub fn from_detail(crate_name: String, detail: &CrateDetail) -> Self {
        let features = detail
            .features
            .iter()
            .filter(|f| f.name != "default")
            .map(|f| FeatureItem {
                name: f.name.clone(),
                selected: false,
            })
            .collect();

        Self {
            crate_name,
            features,
            no_default_features: false,
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        1 + self.features.len()
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.len() > 0 {
            self.cursor = (self.cursor + 1).min(self.len() - 1);
        }
    }

    pub fn toggle_current(&mut self) {
        if self.cursor == 0 {
            self.no_default_features = !self.no_default_features;
        } else if let Some(item) = self.features.get_mut(self.cursor - 1) {
            item.selected = !item.selected;
        }
    }

    pub fn selected_features(&self) -> Vec<String> {
        self.features
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.name.clone())
            .collect()
    }
}

#[derive(PartialEq)]
pub enum Focus {
    Input,
    Results,
    FeatureSelect,
}

pub struct App {
    pub input: String,
    pub cursor_pos: usize,
    pub focus: Focus,
    pub results: Vec<CrateInfo>,
    pub selected: usize,
    pub is_searching: bool,
    pub detail: Option<Result<CrateDetail, String>>,
    pub is_loading_detail: bool,
    pub pending_feature_select: bool,
    pub feature_select: Option<FeatureSelectState>,
    pub is_adding: bool,
    pub add_result: Option<Result<String, String>>,
    pub in_rust_project: bool,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            focus: Focus::Input,
            results: Vec::new(),
            selected: 0,
            is_searching: false,
            detail: None,
            is_loading_detail: false,
            pending_feature_select: false,
            feature_select: None,
            is_adding: false,
            add_result: None,
            in_rust_project: false,
            error: None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self
                .input
                .char_indices()
                .rev()
                .find(|(i, _)| *i < self.cursor_pos)
                .map(|(i, c)| (i, c.len_utf8()));
            if let Some((byte_idx, char_len)) = prev {
                self.input.drain(byte_idx..byte_idx + char_len);
                self.cursor_pos = byte_idx;
            }
        }
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected = (self.selected + 1).min(self.results.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn enter_feature_select(&mut self) {
        if let Some(Ok(detail)) = &self.detail {
            let crate_name = self
                .results
                .get(self.selected)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            self.feature_select = Some(FeatureSelectState::from_detail(crate_name, detail));
            self.focus = Focus::FeatureSelect;
        }
    }
}
