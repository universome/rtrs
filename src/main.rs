use std::time::{Instant};

extern crate rayon;
extern crate nannou;
extern crate derive_more;
#[macro_use] extern crate itertools;

use nannou::prelude::*;
#[macro_use] extern crate float_cmp;

mod render;
mod scene;
mod basics;
mod surface;
mod matrix;

use basics::*;
use render::{RenderOptions, CameraOptions};
use scene::{ProjectionType};
use matrix::{Transformation, Mat3};

static WIDTH: u32 = 640;
static HEIGHT: u32 = 480;
// static WIDTH: u32 = 800;
// static HEIGHT: u32 = 600;
// static WIDTH: u32 = 1280;
// static HEIGHT: u32 = 960;
static mut NUM_FRAMES_SINCE_LAST_SEC: u32 = 0;
static mut LAST_SEC: u32 = 0;


struct Model {
    opts: RenderOptions,
}


fn main() {
    // rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();

    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().event(update).size(WIDTH, HEIGHT).view(view).build().unwrap();

    Model {
        opts: RenderOptions {
            projection_type: ProjectionType::Perspective,
            number_of_lights: 1,
            camera_options: CameraOptions {
                yaw: -0.5 * std::f32::consts::PI,
                pitch: 0.0,
                position: Vec3 {x: 0.0, y: 0.0, z: -1.0},
            },
            specular_strength: 0.0,
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
                Key::W => {
                    model.opts.camera_options.position.z += 0.2;
                },
                Key::S => {
                    model.opts.camera_options.position.z -= 0.2;
                },
                Key::D => {
                    model.opts.camera_options.position.x += 0.2;
                },
                Key::A => {
                    model.opts.camera_options.position.x -= 0.2;
                },
                Key::Up => {
                    model.opts.camera_options.pitch -= 0.1;
                },
                Key::Down => {
                    model.opts.camera_options.pitch += 0.1;
                },
                Key::Right => {
                    model.opts.camera_options.yaw += 0.1;
                },
                Key::Left => {
                    model.opts.camera_options.yaw -= 0.1;
                },
                // Key::Z => {
                //     model.opts.camera_z_position = if model.opts.camera_z_position == -1.0 { -5.0 } else { -1.0 };
                // },
                // Key::S => {
                //     model.opts.specular_strength = if model.opts.specular_strength == 0.0 { 0.5 } else { 0.0 };
                // },
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
    let start = Instant::now();
    let img = render::render(WIDTH, HEIGHT, &model.opts);
    let duration = start.elapsed();
    println!("Frame rendering time: {:?}", duration);

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
            println!("FPS: {}", NUM_FRAMES_SINCE_LAST_SEC + 1);

            LAST_SEC = app.time.floor() as u32;
            NUM_FRAMES_SINCE_LAST_SEC = 0;
        } else {
            NUM_FRAMES_SINCE_LAST_SEC += 1;
        }
    }

    // println!("Time: {}", app.time);

    draw.to_frame(app, &frame).unwrap();
}
