use super::FocusPanel;

pub struct FocusManager {
    pub current_panel: FocusPanel,
    pub panel_history: Vec<FocusPanel>,
    pub focus_ring: [FocusPanel; 3],
    current_index: usize,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current_panel: FocusPanel::Sidebar,
            panel_history: Vec::new(),
            focus_ring: [FocusPanel::Sidebar, FocusPanel::Content, FocusPanel::Detail],
            current_index: 0,
        }
    }
    
    pub fn next_focus(&mut self) {
        self.panel_history.push(self.current_panel);
        self.current_index = (self.current_index + 1) % self.focus_ring.len();
        self.current_panel = self.focus_ring[self.current_index];
    }
    
    pub fn prev_focus(&mut self) {
        self.panel_history.push(self.current_panel);
        self.current_index = if self.current_index == 0 {
            self.focus_ring.len() - 1
        } else {
            self.current_index - 1
        };
        self.current_panel = self.focus_ring[self.current_index];
    }
    
    pub fn set_focus(&mut self, panel: FocusPanel) {
        if self.current_panel != panel {
            self.panel_history.push(self.current_panel);
            self.current_panel = panel;
            
            // 更新索引
            for (i, &p) in self.focus_ring.iter().enumerate() {
                if p == panel {
                    self.current_index = i;
                    break;
                }
            }
        }
    }
    
    pub fn get_current_panel(&self) -> FocusPanel {
        self.current_panel
    }
}