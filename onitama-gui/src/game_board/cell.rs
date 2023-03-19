use egui::{Color32, Stroke, Ui, Widget};

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
}

impl Widget for Cell {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let Cell {
            row,
            col,
            bg_fill,
            size,
        } = self;
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:
        let desired_size = egui::vec2(size, size);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // 3. Interact: Time to check for clicks!
        if response.clicked() {
            tracing::info!("Cell at ({},{})", row, col);
            response.mark_changed(); // report back that the value changed
        }

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Button));

        // 4. Paint!
        // Make sure we need to paint:
        if ui.is_rect_visible(rect) {
            // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
            let stroke: Stroke = (0.5, Color32::BLACK).into();
            ui.painter().rect(rect, 0., bg_fill, stroke);
        }

        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        response
    }
}
