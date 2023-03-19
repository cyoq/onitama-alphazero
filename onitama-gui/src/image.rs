use std::path::PathBuf;

use egui_extras::RetainedImage;

pub struct Image {
    pub name: String,
    pub image: RetainedImage,
}

impl Image {
    pub fn load_image(name: String, path: &PathBuf) -> Self {
        let image_bytes = std::fs::read(path).expect(&format!(
            "Image on path {:?} does not exist!",
            path.as_os_str()
        ));
        let image = RetainedImage::from_svg_bytes(&name, &image_bytes)
            .expect("Image was not loaded successfully!");
        Self { name, image }
    }
}
