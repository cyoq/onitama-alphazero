use egui::{Color32, Painter, Rect, Stroke, Ui, Vec2};

enum TextDirection<'a> {
    Left(&'a str),
    Bottom(&'a str),
    Both(&'a str, &'a str),
    None,
}

const TEXT_FIELD_PADDING: f32 = 15.;

/// A container that allows to hold objects inside of it.
pub struct Cell {
    pub row: u32,
    pub col: u32,
    pub size: f32,
    pub bg_fill: Color32,
}

impl Cell {
    pub fn new(row: u32, col: u32, bg_fill: Color32, size: f32) -> Self {
        Self {
            row,
            col,
            bg_fill,
            size,
        }
    }

    pub fn show<R>(&mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui, Rect) -> R) -> R {
        let text_direction = match (self.row, self.col) {
            (0, 0) => TextDirection::Left("5"),
            (1, 0) => TextDirection::Left("4"),
            (2, 0) => TextDirection::Left("3"),
            (3, 0) => TextDirection::Left("2"),
            (4, 0) => TextDirection::Both("1", "A"),
            (4, 1) => TextDirection::Bottom("B"),
            (4, 2) => TextDirection::Bottom("C"),
            (4, 3) => TextDirection::Bottom("D"),
            (4, 4) => TextDirection::Bottom("E"),
            _ => TextDirection::None,
        };

        let desired_size = egui::vec2(self.size, self.size);
        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // 3. Interact: Time to check for clicks!
        if response.clicked() {
            tracing::debug!("Cell at ({},{})", self.row, self.col);
            response.mark_changed(); // report back that the value changed
        }

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Button));

        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let stroke: Stroke = (0.5, Color32::BLACK).into();
        ui.painter().rect(rect, 0., self.bg_fill, stroke);

        // Draw text near cells if necessary
        match text_direction {
            TextDirection::None => (),
            TextDirection::Left(num) => {
                let offset = egui::Vec2::new(-self.size / 2. - TEXT_FIELD_PADDING - 5., 0.);
                draw_text(ui.painter(), &rect, num, offset);
            }
            TextDirection::Bottom(ch) => {
                let offset = egui::Vec2::new(0., self.size / 2. + TEXT_FIELD_PADDING + 5.);
                draw_text(ui.painter(), &rect, ch, offset);
            }
            TextDirection::Both(num, ch) => {
                let offset = egui::Vec2::new(-self.size / 2. - TEXT_FIELD_PADDING - 5., 0.);
                draw_text(ui.painter(), &rect, num, offset);
                let offset = egui::Vec2::new(0., self.size / 2. + TEXT_FIELD_PADDING + 5.);
                draw_text(ui.painter(), &rect, ch, offset);
            }
        }

        let pos = rect.min;
        let child_rect = Rect::from_min_max(pos, egui::pos2(pos.x + self.size, rect.max.y));

        let mut child_ui = ui.child_ui(child_rect, *ui.layout());
        let result = add_contents(&mut child_ui, rect);
        result
    }
}

fn draw_text(painter: &Painter, rect: &Rect, text: &str, offset: Vec2) {
    let text_coords = rect.center() + offset;
    painter.text(
        text_coords,
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId {
            size: 18.,
            family: egui::FontFamily::Proportional,
        },
        Color32::BLACK,
    );
}
