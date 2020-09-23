//! A simple as possible example demonstrating how to use the `draw` API to display a texture.
extern crate nannou;
use nannou::prelude::*;
use nannou::image;
use nannou::wgpu::{Texture};

use nannou::image::{DynamicImage, ImageBuffer, Rgb};
fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    // Create a new window!
    app.new_window().size(512, 512).view(view).build().unwrap();
    // Load the image from disk and upload it to a GPU texture.
    // let assets = app.assets_path().unwrap();
    // let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    // let texture = wgpu::Texture::from_path(app, img_path).unwrap();
    let mut img = ImageBuffer::from_fn(512, 512, |x, y| {
        if x % 4 == 0 {
            image::Rgb([0u8, 0u8, 0u8])
        } else {
            image::Rgb([0u8, 0u8, 255u8])
        }
    });
    let im = DynamicImage::ImageRgb8(img);
    let texture = wgpu::Texture::from_image(app, &im);
    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    draw.texture(&model.texture);

    println!("draw!");

    draw.to_frame(app, &frame).unwrap();
}

// use nannou::prelude::*;


// fn main() {
//     nannou::sketch(view).run();
// }

// fn view(app: &App, frame: Frame) {
//     let mut img = ImageBuffer::from_fn(512, 512, |x, y| {
//         if x % 2 == 0 {
//             image::Rgb([0u8, 0u8, 0u8])
//         } else {
//             image::Rgb([255u8, 255u8, 255u8])
//         }
//     });
//     // Prepare to draw.
//     let width = 500;
//     let height = 500;
//     let draw = app.draw();
//     let win = app.main_window();
//     win.set_inner_size_pixels(width, height);
//     let t = app.duration.since_start.secs() as f32;
//     // let diagonal = win.top_left().distance(win.bottom_right());

//     // Clear the background to pink.
//     draw.background().color(BLUE);

//     // let max_weight = (1.0 / n as f32) * win.w();
//     // let dn = d / win.w();
//     // let weight = max_weight * dn;
//     println!("{:?}", t);

//     let tex = Texture::from_image(app, &img);
//     draw.texture(&tex);

//     let mut c: u32 = 0;

//     for w in 0..width {
//         for h in 0..height {
//             c += w + h;
//             // draw.line()
//             //     // .weight(weight)
//             //     .points(pt2(w as f32, h as f32), pt2(w as f32 + 1.0, h as f32 + 1.0))
//             //     .rgb(0.1, 0.9, 0.1);
//                 // .hsla(hue, 1.0, 1.0, dn);
//         }
//     }
//     // for i in 0..n {
//     //     let f = i as f32 / n as f32;
//     //     let max_weight = (1.0 / n as f32) * win.w();
//     //     let x = win.x.lerp(f);
//     //     let hz = 0.125;
//     //     let tx = (t * hz * 2.0 * PI).sin() * win.right();
//     //     let d = (tx - x).abs();
//     //     let dn = d / win.w();
//     //     let weight = max_weight * dn;
//     //     let hue = 1.0;

//     //     // Linear.
//     //     // let pa = pt2(x, win.top());
//     //     // let pb = pt2(x, win.bottom());

//     //     // Radial.
//     //     let rad = (t * 0.05 + f) * 2.0 * PI;
//     //     let mag = diagonal;
//     //     let pa = pt2(rad.cos() * mag, rad.sin() * mag);
//     //     let pb = pt2(rad.cos() * -mag, rad.sin() * -mag);

//     //     //let hue = t * 0.1 + dn * 0.3;
//     // }

//     // Write to the window frame.
//     draw.to_frame(app, &frame).unwrap();
// }
