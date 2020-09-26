//! A simple as possible example demonstrating how to use the `draw` API to display a texture.
extern crate nannou;
extern crate derive_more;

use nannou::prelude::*;
#[macro_use] extern crate float_cmp;

mod render;
mod scene;
mod basics;
mod surface;

use render::{RenderOptions};
use scene::{ProjectionType};

static WIDTH: u32 = 640;
static HEIGHT: u32 = 480;
// static WIDTH: u32 = 1280;
// static HEIGHT: u32 = 960;
static mut NUM_FRAMES_SINCE_LAST_SEC: u32 = 0;
static mut LAST_SEC: u32 = 0;


struct Model {
    opts: RenderOptions,
}


fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().event(update).size(WIDTH, HEIGHT).view(view).build().unwrap();

    Model {
        opts: RenderOptions {
            projection_type: ProjectionType::Perspective,
            number_of_lights: 1,
            camera_z_position: -1.0,
        }
    }
}


fn update(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyReleased(key) => {
            match key {
                Key::L => {
                    model.opts.number_of_lights = model.opts.number_of_lights % 2 + 1;
                },
                Key::P => {
                    model.opts.projection_type = match model.opts.projection_type {
                        ProjectionType::Parallel => ProjectionType::Perspective,
                        ProjectionType::Perspective => ProjectionType::Parallel,
                    };
                },
                Key::Z => {
                    model.opts.camera_z_position = if model.opts.camera_z_position == -1.0 { -10.0 } else { -1.0 };
                },
                _ => {},
            }
        }
        // MousePressed(_button) => {
        //     println!("global scope: GLOBAL = {}", GLOBAL);
        // }
        _ => (),
    }
}


fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let img = render::render(WIDTH, HEIGHT, &model.opts);

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
