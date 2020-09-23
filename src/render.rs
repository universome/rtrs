use nannou::image::{DynamicImage, ImageBuffer, Rgb};


struct FloatPixel {r: f32, g: f32, b: f32}

pub fn render() -> DynamicImage {
    let image_buf = ImageBuffer::from_fn(512, 512, compute_pixel);

    DynamicImage::ImageRgb16(image_buf)
}


impl From<FloatPixel> for Rgb<u16> {
    fn from(pixel: FloatPixel) -> Self {
        Rgb([
            (pixel.r * (u16::MAX - 1) as f32 ) as u16,
            (pixel.g * (u16::MAX - 1) as f32 ) as u16,
            (pixel.b * (u16::MAX - 1) as f32 ) as u16,
        ])
    }
}

fn compute_pixel(x: u32, y: u32) -> Rgb<u16> {
    let pixel;

    if x % 10 < 5 && y % 10 < 5 {
        pixel = FloatPixel {r: 0.0, g: 1.0, b: 0.0};
    } else {
        pixel = FloatPixel {r: 0.0, g: 0.0, b: 1.0}
    }

    pixel.into()
}
