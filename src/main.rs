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
mod basics;
mod surface;
mod matrix;

use crate::scene::*;
use crate::surface::*;
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
    camera_options: CameraOptions,
    selected_pixel: Option<(u32, u32)>,
    selected_object_idx: Option<usize>,
    transformations: [Transformation; 2],
    specular_strengths: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct CameraOptions {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
}


fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window().event(update).size(WIDTH, HEIGHT).view(view).build().unwrap();

    build_model()
}


fn build_model() -> Model {
    let render_options = RenderOptions::defaults();

    Model {
        opts: render_options,
        is_mouse_inited: false,
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        mouse_sensitivity: 0.001,
        move_speed: 0.5,
        mouse_is_in_window: false,
        scene: setup_scene(RenderOptions::defaults())
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
                Key::W => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[2] * -model.move_speed),
                Key::S => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[2] * model.move_speed),
                Key::D => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[0] * model.move_speed),
                Key::A => model.opts.camera_options.position = &model.opts.camera_options.position + &(&camera_transformation.transform_mat[0] * -model.move_speed),
                Key::Left => model.opts.camera_options.yaw += 0.1,
                Key::Right => model.opts.camera_options.yaw -= 0.1,
                Key::Q => *model = build_model(), // Reset
                _ => {},
            }
        },
        MouseMoved(point) => {
            if !model.mouse_is_in_window {
                println!("Mouse is not in window!");
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
            model.opts.camera_options.yaw += offset_x;
            model.opts.camera_options.pitch += offset_y;

            model.opts.camera_options.pitch = model.opts.camera_options.pitch
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
        },
        MousePressed(button) => {
            if button == MouseButton::Left {
                model.opts.selected_pixel = Some((
                    (model.curr_mouse_x + (WIDTH as f32) / 2.0) as u32,
                    (model.curr_mouse_y + (HEIGHT as f32) / 2.0) as u32,
                ));
            }
        },
        _ => (),
    }

    if let Some(idx) = model.opts.selected_object_idx {
        model.opts.specular_strengths[idx] = 1.0;
    }
}


fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let start = Instant::now();
    let img = render_model(model);
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


pub fn render_model(model: &Model) -> DynamicImage {
    let scene = setup_scene(model.opts.clone());
    let pixels = iproduct!(0..HEIGHT, 0..WIDTH)
        .collect::<Vec<(u32, u32)>>()
        .par_iter()
        .map(|p: &(u32, u32)| -> Color {
            scene.compute_pixel(p.1, HEIGHT - p.0)
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


fn setup_scene(render_options: RenderOptions) -> Scene {
    let mut lights = vec![Light {
        location: Point {x: 0.0, y: 5.0, z: 0.0},
        color: Color {r: 1.0, g: 1.0, b: 1.0},
    }];

    // if options.number_of_lights == 2 {
    //     lights.push(Light {
    //         location: Point {x: -3.0, y: 0.0, z: 0.0},
    //         color: Color {r: 1.0, g: 1.0, b: 1.0},
    //     });
    // }

    let lookat_transform = Transformation::create_look_at(
        &render_options.camera_options.position,
        render_options.camera_options.yaw,
        render_options.camera_options.pitch,
    );

    // let sphere = Sphere {
    //     center: Point {x: 1.0, y: -1.5, z: -0.5},
    //     radius: 0.5,
    //     color: Color {r: 0.0, g: 0.0, b: 1.0},
    //     specular_strength: render_options.specular_strength,
    // };
    // let ellipsoid = Ellipsoid {
    //     center: Point {x: 0.0, y: 0.0, z: 0.0},
    //     color: Color {r: 1.0, g: 0.0, b: 0.0},
    //     specular_strength: render_options.specular_strength,
    //     scale: DiagMat3 {a: 0.35, b: 0.25, c: 0.25}
    // };
    let sphere = Sphere {
        center: Point {x: 0.0, y: 0.0, z: 0.0},
        radius: 1.0,
        color: Color {r: 0.0, g: 0.0, b: 1.0},
        specular_strength: render_options.specular_strengths[0],
    };
    let sphere_transform = &lookat_transform * &render_options.transformations[0];
    let transformed_sphere_a = TransformedSurface::new(sphere_transform, sphere);

    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
    let transformed_plane = TransformedSurface::new(lookat_transform.clone(), plane);

    let sphere_b = Sphere {
        center: Point {x: -1.0, y: 1.0, z: 0.0},
        radius: 0.25,
        color: Color {r: 1.0, g: 0.0, b: 0.0},
        specular_strength: render_options.specular_strengths[1],
    };
    let sphere_b_transform = &lookat_transform * &render_options.transformations[1];
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
            Box::new(transformed_sphere_a),
            Box::new(transformed_sphere_b),
            Box::new(transformed_plane),
        ],
        camera: Camera::from_z_position(-1.0, render_options.projection_type, WIDTH, HEIGHT),
        background_color: Color {r: 0.204, g: 0.596, b: 0.86},
        lights: lights,
        ambient_strength: 0.3,
        diffuse_strength: 0.7,
    }

    // if let Some((i, j)) = options.selected_pixel {
    //     if let Some((obj, _)) = scene.get_object_at_pixel(i, j) {
    //         if ptr::eq(obj, &transformed_sphere_a) {
    //             options.selected_object_idx = Some(0);
    //         } else if ptr::eq(obj, &transformed_sphere_b) {
    //             options.selected_object_idx = Some(1);
    //         } else {
    //             options.selected_object_idx = None;
    //         }
    //     }
    // }
}


impl RenderOptions {
    fn defaults() -> Self {
        RenderOptions {
            projection_type: ProjectionType::Perspective,
            number_of_lights: 1,
            camera_options: CameraOptions {
                yaw: -0.5 * std::f32::consts::PI,
                pitch: 0.0,
                position: Vec3 {x: 0.0, y: 0.0, z: -10.0},
            },
            selected_pixel: None,
            selected_object_idx: None,
            specular_strengths: [0.0, 0.0],
            transformations: [
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
                        Vec3 {x: 0.0, y: 1.0, z: 0.0},
                        Vec3 {x: 0.0, y: 0.0, z: 1.0},
                    ]},
                    translation: Vec3 {x: -1.0, y: 1.0, z: 0.0},
                }
            ],
        }
    }
}
