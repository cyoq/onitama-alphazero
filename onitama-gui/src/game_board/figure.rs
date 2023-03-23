use egui::*;
use egui_extras::RetainedImage;

pub struct Figure<'a> {
    pub outer_rect: &'a Rect,
    pub image: &'a RetainedImage,
}

impl<'a> Widget for Figure<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let Figure {
            outer_rect: rect,
            image,
        } = self;
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        let inner_rect = rect.shrink(20.);
        let response = ui.allocate_rect(inner_rect, Sense::click());

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Button));

        // 4. Paint!
        // Make sure we need to paint:
        if !ui.is_rect_visible(inner_rect) {
            return response;
        }

        let texture = image.texture_id(ui.ctx());
        ui.painter().image(
            texture,
            inner_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        response
    }
}
