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
    is_mouse_inited: bool,
    curr_mouse_x: f32,
    curr_mouse_y: f32,
    mouse_sensitivity: f32,
}


fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

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
        },
        is_mouse_inited: false,
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        mouse_sensitivity: 0.01,
    }
}


fn update(_app: &App, model: &mut Model, event: WindowEvent) {
    let camera_transformation = Transformation::create_look_at(
        &model.opts.camera_options.position,
        model.opts.camera_options.yaw,
        model.opts.camera_options.pitch,
    );

    match event {
        KeyPressed(key) => {
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
                Key::W => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[2] * -0.1),
                Key::S => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[2] * 0.1),
                Key::D => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[0] * 0.1),
                Key::A => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[0] * -0.1),
                Key::Left => model.opts.camera_options.yaw += 0.1,
                Key::Right => model.opts.camera_options.yaw -= 0.1,
                _ => {},
            }
        },
        MouseMoved(point) => {
            if (!model.is_mouse_inited) {
                model.curr_mouse_x = point.x;
                model.curr_mouse_y = point.y;
                model.is_mouse_inited = true;
            }

            let offset_x = (point.x - model.curr_mouse_x) * model.mouse_sensitivity;
            let offset_y = (model.curr_mouse_y - point.y) * model.mouse_sensitivity;

            model.curr_mouse_x = point.x;
            model.curr_mouse_y = point.y;
            model.opts.camera_options.yaw += offset_x;
            model.opts.camera_options.pitch += offset_y;

            model.opts.camera_options.pitch = model.opts.camera_options.pitch
                .min(0.5 * std::f32::consts::PI - 0.001)
                .max(-0.5 * std::f32::consts::PI + 0.001);
        },
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
