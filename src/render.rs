use nannou::image::{DynamicImage, ImageBuffer, Rgb};

use crate::scene::{Scene, ViewingPlane, Sphere, Camera, FloatPixel, Point, Vec3};


pub fn render() -> DynamicImage {
    let scene = Scene {
        objects: vec![&Sphere {
            center: Point {x: 0.0, y: 0.0, z: 0.0},
            radius: 0.8,
            color: FloatPixel {r: 1.0, g: 0.0, b: 0.0},
        }],
        camera: Camera::from_z_position(-1.0),
        viewing_plane: ViewingPlane {
            z: 0.0,
            x_min: -2.0,
            x_max: 2.0,
            y_min: -1.5,
            y_max: 1.5,
            width: 640,
            height: 480,
        },
        background_color: FloatPixel { r: 0.2, g: 0.5, b: 0.2}
    };

    let image_buf = ImageBuffer::from_fn(640, 480, |i: u32, j: u32| -> Rgb<u16> {
        scene.compute_pixel(i, j).into()
    });

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
