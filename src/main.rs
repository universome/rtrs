//! A simple as possible example demonstrating how to use the `draw` API to display a texture.
extern crate nannou;
extern crate derive_more;

use nannou::prelude::*;
#[macro_use] extern crate float_cmp;

mod render;
mod scene;

static mut NUM_FRAMES_SINCE_LAST_SEC: u32 = 0;
static mut LAST_SEC: u32 = 0;

fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 480).view(view).build().unwrap();

    let img = render::render();
    let texture = wgpu::Texture::from_image(app, &img);

    Model { texture }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();

    draw.texture(&model.texture);

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

    draw.to_frame(app, &frame).unwrap();
}
