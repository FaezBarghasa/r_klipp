//! CNC Tool Length (G43/G44/G49) & Cutter Radius (G41/G42) Compensation.
//!
//! Applies dynamic offsets to Cartesian coordinates based on active tool index in the tool table.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolEntry {
    pub tool_id: u8,
    pub length_offset_mm: f64,
    pub diameter_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutterRadiusCompMode {
    Off,   // G40
    Left,  // G41 (Climb milling profile left)
    Right, // G42 (Conventional milling profile right)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolTableManager {
    pub active_tool_id: u8,
    pub length_comp_enabled: bool,
    pub radius_comp_mode: CutterRadiusCompMode,
    pub tools: [ToolEntry; 16],
}

impl ToolTableManager {
    pub fn new() -> Self {
        let default_tool = ToolEntry {
            tool_id: 0,
            length_offset_mm: 0.0,
            diameter_mm: 0.0,
        };

        Self {
            active_tool_id: 0,
            length_comp_enabled: false,
            radius_comp_mode: CutterRadiusCompMode::Off,
            tools: [default_tool; 16],
        }
    }

    /// Sets tool geometry parameters in the tool table
    pub fn set_tool(&mut self, slot: usize, tool: ToolEntry) {
        if slot < self.tools.len() {
            self.tools[slot] = tool;
        }
    }

    /// Selects active tool (`T<n> M6`)
    pub fn select_tool(&mut self, tool_id: u8) {
        self.active_tool_id = tool_id;
    }

    /// Enables Tool Length Offset (`G43 H<n>`)
    pub fn enable_length_comp(&mut self, enabled: bool) {
        self.length_comp_enabled = enabled;
    }

    /// Returns the active tool length offset (applied to Z axis)
    pub fn active_length_offset(&self) -> f64 {
        if !self.length_comp_enabled {
            return 0.0;
        }
        for tool in &self.tools {
            if tool.tool_id == self.active_tool_id {
                return tool.length_offset_mm;
            }
        }
        0.0
    }

    /// Applies length compensation to target Z coordinate
    pub fn apply_z_comp(&self, target_z: f64) -> f64 {
        target_z + self.active_length_offset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_length_offset_application() {
        let mut manager = ToolTableManager::new();
        // Tool 1: 50mm endmill
        manager.set_tool(1, ToolEntry {
            tool_id: 1,
            length_offset_mm: 50.0,
            diameter_mm: 6.0,
        });

        manager.select_tool(1);
        assert_eq!(manager.apply_z_comp(10.0), 10.0); // G49 (comp off)

        manager.enable_length_comp(true); // G43 H1
        assert_eq!(manager.apply_z_comp(10.0), 60.0); // 10mm + 50mm tool offset
    }
}
