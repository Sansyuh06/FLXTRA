//! Display list for rendering
use aegis_css::values::Color;
use aegis_layout::box_model::Rect;

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    SolidColor { color: Color, rect: Rect },
    Text { text: String, x: f32, y: f32, color: Color, size: f32 },
    Border { rect: Rect, color: Color, width: f32 },
    Image { url: String, rect: Rect },
}

#[derive(Debug, Default)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
}

impl DisplayList {
    pub fn new() -> Self { Self::default() }
    
    pub fn push(&mut self, cmd: DisplayCommand) { self.commands.push(cmd); }
    
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.push(DisplayCommand::SolidColor { color, rect });
    }
    
    pub fn draw_text(&mut self, text: String, x: f32, y: f32, color: Color, size: f32) {
        self.push(DisplayCommand::Text { text, x, y, color, size });
    }
    
    pub fn draw_border(&mut self, rect: Rect, color: Color, width: f32) {
        self.push(DisplayCommand::Border { rect, color, width });
    }
}
