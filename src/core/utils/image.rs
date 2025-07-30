use actix_multipart::form::tempfile::TempFile;
use actix_web::mime;

pub fn get_image_type(file: &TempFile) -> Option<image::ImageFormat> {
    if let Some(mime_type) = &file.content_type {
        match mime_type.type_() {
            mime::PNG => Some(image::ImageFormat::Png),
            mime::JPEG => Some(image::ImageFormat::Jpeg),
            mime::GIF => Some(image::ImageFormat::Gif),
            _ => None,
        }
    } else {
        None
    }
}
