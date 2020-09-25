//! A simple as possible example demonstrating how to use the `draw` API to display a texture.
extern crate nannou;
extern crate derive_more;

use nannou::prelude::*;
#[macro_use] extern crate float_cmp;

mod render;
mod scene;

use scene::{ProjectionType};

static WIDTH: u32 = 640;
static HEIGHT: u32 = 480;
// static WIDTH: u32 = 1280;
// static HEIGHT: u32 = 960;
static mut NUM_FRAMES_SINCE_LAST_SEC: u32 = 0;
static mut LAST_SEC: u32 = 0;


fn main() {
    nannou::app(model).run();
}

// struct Model {
//     texture: wgpu::Texture,
// }
struct Model {}

fn model(app: &App) -> Model {
    app.new_window().size(WIDTH, HEIGHT).view(view).build().unwrap();

    Model {}
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let img = render::render(WIDTH, HEIGHT, ProjectionType::Perspective);

    unsafe {
        if NUM_FRAMES_SINCE_LAST_SEC == 0 && LAST_SEC % 10 == 0 {
           img.save("image.tga").unwrap();
        }
    }

    let texture = wgpu::Texture::from_image(app, &img);

    draw.texture(&texture);

    // TODO: there must be some event that we can subscribe on
    // which would allow us to get rid of mutable statics
    unsafe {
        if app.time.floor() as u32 != LAST_SEC {
            println!("FPS: {}", NUM_FRAMES_SINCE_LAST_SEC);

            LAST_SEC = app.time.floor() as u32;
            NUM_FRAMES_SINCE_LAST_SEC = 0;
        } else {
            NUM_FRAMES_SINCE_LAST_SEC += 1;
        }
    }

    println!("Time: {}", app.time);

    draw.to_frame(app, &frame).unwrap();
}
