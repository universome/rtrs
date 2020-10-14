use std::ptr;
use std::time::{Instant};

extern crate rayon;
extern crate nannou;
extern crate derive_more;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate float_cmp;

use rayon::prelude::*;
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};

mod scene;
mod camera;
mod basics;
mod surface;
mod matrix;

use crate::scene::Scene;
use crate::camera::{Camera, ProjectionType};
use crate::surface::{TransformedSurface, Sphere, Plane, Cone};
use crate::basics::*;
use crate::matrix::{Mat3, Transformation};

static WIDTH: u32 = 640;
static HEIGHT: u32 = 480;
// static WIDTH: u32 = 800;
// static HEIGHT: u32 = 600;
// static WIDTH: u32 = 1280;
// static HEIGHT: u32 = 960;
static mut NUM_FRAMES_SINCE_LAST_SEC: u32 = 0;
static mut LAST_SEC: u32 = 0;


pub struct Model {
    opts: RenderOptions,
    scene: Scene,
    is_mouse_inited: bool,
    curr_mouse_x: f32,
    curr_mouse_y: f32,
    mouse_sensitivity: f32,
    move_speed: f32,
    mouse_is_in_window: bool,
}


#[derive(Debug, Clone)]
pub struct RenderOptions {
    projection_type: ProjectionType,
    number_of_lights: u32,
    camera_opts: CameraOptions,
    selected_pixel: Option<(u32, u32)>,
    selected_object_idx: Option<usize>,
    transformations: [Transformation; 3],
    specular_strengths: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct CameraOptions {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
}


fn main() {
    // rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();
    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().event(update).size(WIDTH, HEIGHT).view(view).build().unwrap();

    build_model()
}


fn update(_app: &App, model: &mut Model, event: WindowEvent) {
    let camera_transformation = Transformation::create_look_at(
        &model.opts.camera_opts.position,
        model.opts.camera_opts.yaw,
        model.opts.camera_opts.pitch,
    );

    // println!("{:?}", _app.mouse);

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
                Key::W => model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * -model.move_speed),
                Key::S => model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * model.move_speed),
                Key::D => model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * model.move_speed),
                Key::A => model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * -model.move_speed),
                Key::Up => {
                    if let Some(idx) = model.opts.selected_object_idx {
                        // println!("{:?}", camera_transformation.transform_mat[1]);
                        model.opts.transformations[idx].translation = &model.opts.transformations[idx].translation + &(&camera_transformation.transform_mat[1] * model.move_speed);
                    }
                },
                Key::Down => {
                    if let Some(idx) = model.opts.selected_object_idx {
                        model.opts.transformations[idx].translation = &model.opts.transformations[idx].translation + &(&camera_transformation.transform_mat[1] * -model.move_speed);
                    }
                },
                Key::Right => {
                    if let Some(idx) = model.opts.selected_object_idx {
                        model.opts.transformations[idx].translation = &model.opts.transformations[idx].translation + &(&camera_transformation.transform_mat[0] * model.move_speed);
                    }
                },
                Key::Left => {
                    if let Some(idx) = model.opts.selected_object_idx {
                        model.opts.transformations[idx].translation = &model.opts.transformations[idx].translation + &(&camera_transformation.transform_mat[0] * -model.move_speed);
                    }
                },
                Key::Q => *model = build_model(), // Reset
                _ => {},
            }
        },
        MouseMoved(point) => {
            if !model.mouse_is_in_window {
                println!("Mouse is not in the window!");
                return;
            }
            if !model.is_mouse_inited {
                model.curr_mouse_x = point.x;
                model.curr_mouse_y = point.y;
                model.is_mouse_inited = true;
            }

            let offset_x = (point.x - model.curr_mouse_x) * model.mouse_sensitivity;
            let offset_y = (model.curr_mouse_y - point.y) * model.mouse_sensitivity;

            model.curr_mouse_x = point.x;
            model.curr_mouse_y = point.y;
            model.opts.camera_opts.yaw += offset_x;
            model.opts.camera_opts.pitch += offset_y;

            model.opts.camera_opts.pitch = model.opts.camera_opts.pitch
                .min(0.5 * std::f32::consts::PI - 0.001)
                .max(-0.5 * std::f32::consts::PI + 0.001);
        },
        MouseEntered => {
            model.mouse_is_in_window = true;
            model.is_mouse_inited = false;
        },
        MouseExited => {
            model.mouse_is_in_window = false;
            model.is_mouse_inited = false;
            model.opts.selected_pixel = None;
            model.opts.specular_strengths = [0.0, 0.0, 0.0];
        },
        MousePressed(button) => {
            if button != MouseButton::Left {
                return;
            }

            let i = (model.curr_mouse_x + (WIDTH as f32) / 2.0) as u32;
            let j = (model.curr_mouse_y + (HEIGHT as f32) / 2.0) as u32;

            // model.scene.compute_pixel(i, j, true);

            if let Some(obj_idx) = model.scene.get_object_idx_at_pixel(i, j) {
                model.opts.selected_object_idx = Some(obj_idx);
                model.opts.specular_strengths[obj_idx] = 0.7;
            } else {
                model.opts.selected_object_idx = None;
                model.opts.specular_strengths = [0.0, 0.0, 0.0];
            }

            dbg!(&model.opts.camera_opts.position);
        },
        _ => return,
    }

    model.scene = setup_scene(&model.opts);
}


fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let start = Instant::now();
    let img = render_model(model);
    let duration = start.elapsed();
    // println!("Frame rendering time: {:?}", duration);

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


fn build_model() -> Model {
    let render_options = RenderOptions::defaults();

    Model {
        scene: setup_scene(&render_options),
        opts: render_options,
        is_mouse_inited: false,
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        mouse_sensitivity: 0.001,
        move_speed: 0.5,
        mouse_is_in_window: false,
    }
}


pub fn render_model(model: &Model) -> DynamicImage {
    let pixels = iproduct!(0..HEIGHT, 0..WIDTH)
        .collect::<Vec<(u32, u32)>>()
        .par_iter()
        .map(|p: &(u32, u32)| -> Color {
            model.scene.compute_pixel(p.1, HEIGHT - p.0, false)
        })
        .collect::<Vec<Color>>();

    let mut img = RgbImage::new(WIDTH, HEIGHT);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            img.put_pixel(x, y, pixels[(WIDTH * y + x) as usize].clone().into());
        }
    }

    DynamicImage::ImageRgb8(img)
}


fn setup_scene(render_options: &RenderOptions) -> Scene {
    let lookat_transform = Transformation::create_look_at(
        &render_options.camera_opts.position,
        render_options.camera_opts.yaw,
        render_options.camera_opts.pitch,
    );

    let lights = vec![Light {
        location: &lookat_transform * &Point {x: 0.0, y: 5.0, z: 0.0},
        color: Color {r: 1.0, g: 1.0, b: 1.0},
    }];

    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
    let plane_transform = &lookat_transform * &render_options.transformations[0];
    let transformed_plane = TransformedSurface::new(plane_transform, plane);

    let mut sphere_a = Sphere::new();
    sphere_a.specular_strength = render_options.specular_strengths[1];
    let sphere_a_transform = &lookat_transform * &render_options.transformations[1];
    let transformed_sphere_a = TransformedSurface::new(sphere_a_transform, sphere_a);

    let sphere_b = Sphere {
        center: Point {x: -1.0, y: 1.0, z: 0.0},
        radius: 0.25,
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: render_options.specular_strengths[2],
    };
    let sphere_b_transform = &lookat_transform * &render_options.transformations[2];
    let transformed_sphere_b = TransformedSurface::new(sphere_b_transform, sphere_b);

    // let cone = Cone {
    //     apex: Point {x: 0.0, y: 0.0, z: 0.0},
    //     half_angle: 0.5,
    //     height: 0.5,
    //     color: Color {r: 0.0, g: 1.0, b: 0.0},
    //     specular_strength: render_options.specular_strengths[2],
    // };
    // let cone_transform = &lookat_transform * &render_options.transformations[2];
    // let transformed_cone = TransformedSurface::new(cone_transform, &cone);

    Scene {
        objects: vec![
            Box::new(transformed_plane),
            Box::new(transformed_sphere_a),
            Box::new(transformed_sphere_b),
        ],
        camera: Camera::from_z_position(-1.0, render_options.projection_type, WIDTH, HEIGHT),
        background_color: Color {r: 0.204, g: 0.596, b: 0.86},
        lights: lights,
        ambient_strength: 0.3,
        diffuse_strength: 0.7,
    }
}


impl RenderOptions {
    fn defaults() -> Self {
        RenderOptions {
            projection_type: ProjectionType::Perspective,
            number_of_lights: 1,
            selected_pixel: None,
            selected_object_idx: None,
            specular_strengths: [0.0, 0.0, 0.0],
            camera_opts: CameraOptions {
                yaw: -0.5 * std::f32::consts::PI,
                pitch: 0.0,
                position: Vec3 {x: 0.0, y: 0.0, z: -5.0},
            },
            transformations: [
                Transformation::identity(),
                Transformation {
                    transform_mat: Mat3 {rows: [
                        Vec3 {x: 0.25, y: 0.0, z: 0.0},
                        Vec3 {x: 0.0, y: 0.25, z: 0.0},
                        Vec3 {x: 0.0, y: 0.0, z: 0.25},
                    ]},
                    translation: Vec3 {x: 0.0, y: 0.0, z: 0.0},
                },
                Transformation {
                    transform_mat: Mat3 {rows: [
                        Vec3 {x: 1.0, y: 0.0, z: 0.0},
                        Vec3 {x: 0.0, y: 0.3, z: 0.0},
                        Vec3 {x: 0.0, y: 0.0, z: 1.0},
                    ]},
                    translation: Vec3 {x: -1.0, y: 0.0, z: 0.0},
                }
            ],
        }
    }
}
