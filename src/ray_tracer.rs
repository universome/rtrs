use std::time::{Instant};

use rayon::prelude::*;
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};

use crate::scene::Scene;
use crate::camera::{Camera, ProjectionType};
use crate::surface::{TransformedSurface, Sphere, Plane, Cone};
use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};

// static WIDTH: u32 = 640;
// static HEIGHT: u32 = 480;
static WIDTH: u32 = 800;
static HEIGHT: u32 = 600;
// static WIDTH: u32 = 960;
// static HEIGHT: u32 = 720;
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
    scroll_speed: f32,
    rotation_speed: f32,
    scale_speed: f32,
}


#[derive(Debug, Clone)]
pub struct RenderOptions {
    projection_type: ProjectionType,
    number_of_lights: u32,
    camera_opts: CameraOptions,
    selected_pixel: Option<(u32, u32)>,
    selected_object_idx: Option<usize>,
    transformations: [AffineMat3; 5],
    specular_strengths: [f32; 5],
    spheres_fly_radius: f32,
    spheres_fly_speed: f32,
    fov: f32,
}

#[derive(Debug, Clone)]
pub struct CameraOptions {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
}


pub fn launch() {
    nannou::app(model).event(update_on_event).run();
}


fn model(app: &App) -> Model {
    app
        .new_window()
        .title("CS248 Computer Graphics")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    build_model()
}


fn update_on_event(app: &App, model: &mut Model, event: Event) {
    model.opts.update_transformations_on_time(app.time);
    process_keys(app, model);
    process_mouse_events(app, model, event);
    process_mouse_move(app, model);

    model.scene = setup_scene(&model.opts);
}


fn process_keys(app: &App, model: &mut Model) {
    let camera_transformation = AffineMat3::create_look_at(
        &model.opts.camera_opts.position,
        model.opts.camera_opts.yaw,
        model.opts.camera_opts.pitch,
    );

    if app.keys.down.contains(&Key::W) {
        model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * -model.move_speed);
    }

    if app.keys.down.contains(&Key::S) {
        model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * model.move_speed);
    }

    if app.keys.down.contains(&Key::D) {
        model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * model.move_speed);
    }

    if app.keys.down.contains(&Key::A) {
        model.opts.camera_opts.position = &model.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * -model.move_speed);
    }

    if app.keys.down.contains(&Key::L) {
        model.opts.selected_object_idx = Some(4); // Selecting the light
    }

    if let Some(idx) = model.opts.selected_object_idx {
        let mut transformation = None;

        if app.keys.down.contains(&Key::Key1) {
            if app.keys.down.contains(&Key::Up) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0 + model.scale_speed, 1.0, 1.0)));
            } else if app.keys.down.contains(&Key::Down) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0 - model.scale_speed, 1.0, 1.0)));
            }
        } else if app.keys.down.contains(&Key::Key2) {
            if app.keys.down.contains(&Key::Up) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0 + model.scale_speed, 1.0)));
            } else if app.keys.down.contains(&Key::Down) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0 - model.scale_speed, 1.0)));
            }
        } else if app.keys.down.contains(&Key::Key3) {
            if app.keys.down.contains(&Key::Up) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0, 1.0 + model.scale_speed)));
            } else if app.keys.down.contains(&Key::Down) {
                transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0, 1.0 - model.scale_speed)));
            }
        } else if app.keys.down.contains(&Key::Up) {
            transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[1] * model.move_speed));
        } else if app.keys.down.contains(&Key::Down) {
            transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[1] * -model.move_speed));
        } else if app.keys.down.contains(&Key::Right) {
            transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[0] * model.move_speed));
        } else if app.keys.down.contains(&Key::Left) {
            transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[0] * -model.move_speed));
        } else if app.keys.down.contains(&Key::I) {
            transformation = Some(AffineMat3::rotation(model.rotation_speed, Vec3::new(1.0, 0.0, 0.0)));
        } else if app.keys.down.contains(&Key::O) {
            transformation = Some(AffineMat3::rotation(model.rotation_speed, Vec3::new(0.0, 1.0, 0.0)));
        } else if app.keys.down.contains(&Key::P) {
            transformation = Some(AffineMat3::rotation(model.rotation_speed, Vec3::new(0.0, 0.0, 1.0)));
        }

        if let Some(T) = transformation {
            model.opts.transformations[idx] = &model.opts.transformations[idx] * &T;
        }
    }

    if app.keys.down.contains(&Key::Q) {
        *model = build_model();
    }
}


fn process_mouse_events(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {id: _, simple: window_event } => {
            if window_event.is_none() {
                return;
            }

            match window_event.unwrap() {
                MouseEntered => {
                    println!("Mouse entered!");
                    model.mouse_is_in_window = true;
                    model.is_mouse_inited = false;
                },
                MouseExited => {
                    model.mouse_is_in_window = false;
                    model.is_mouse_inited = false;
                    model.opts.selected_pixel = None;
                    model.opts.specular_strengths = [0.0, 0.0, 0.0, 0.0, 1.0];
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
                        model.opts.specular_strengths = [0.0, 0.0, 0.0, 0.0, 1.0];
                    }

                    // dbg!(&model.opts.camera_opts.position);
                },
                MouseWheel(scroll_delta, _) => {
                    match scroll_delta {
                        MouseScrollDelta::PixelDelta(position) => {
                            model.opts.fov += (position.y as f32) * model.scroll_speed;
                            model.opts.fov = model.opts.fov
                                .min(std::f32::consts::PI * 165.0 / 180.0)
                                .max(std::f32::consts::PI * 15.0 / 180.0);
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }
}


fn process_mouse_move(app: &App, model: &mut Model) {
    if !model.mouse_is_in_window {
        return;
    }

    if !model.is_mouse_inited {
        model.curr_mouse_x = app.mouse.x;
        model.curr_mouse_y = app.mouse.y;
        model.is_mouse_inited = true;
    }

    let offset_x = (app.mouse.x - model.curr_mouse_x) * model.mouse_sensitivity;
    let offset_y = (model.curr_mouse_y - app.mouse.y) * model.mouse_sensitivity;

    model.curr_mouse_x = app.mouse.x;
    model.curr_mouse_y = app.mouse.y;
    model.opts.camera_opts.yaw += offset_x;
    model.opts.camera_opts.pitch += offset_y;

    model.opts.camera_opts.pitch = model.opts.camera_opts.pitch
        .min(0.5 * std::f32::consts::PI - 0.001)
        .max(-0.5 * std::f32::consts::PI + 0.001);

    // (*app.main_window()).set_cursor_position_points(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    // model.curr_mouse_x = app.mouse.x;
    // model.curr_mouse_y = app.mouse.y;
}


fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let start = Instant::now();
    let img = render_model(model);
    let duration = start.elapsed();

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
    println!("Building model!");
    let render_options = RenderOptions::defaults();

    Model {
        scene: setup_scene(&render_options),
        opts: render_options,
        is_mouse_inited: false,
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        mouse_sensitivity: 0.001,
        move_speed: 0.05,
        mouse_is_in_window: false,
        scroll_speed: 0.01,
        rotation_speed: 0.1,
        scale_speed: 0.05,
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
    let lookat_transform = AffineMat3::create_look_at(
        &render_options.camera_opts.position,
        render_options.camera_opts.yaw,
        render_options.camera_opts.pitch,
    );

    let lights = vec![Light {
        location: (&lookat_transform * &render_options.transformations[4].translation).into(),
        color: Color {r: 1.0, g: 1.0, b: 1.0},
    }];

    let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
    let plane_transform = &lookat_transform * &render_options.transformations[0];
    let transformed_plane = TransformedSurface::new(plane_transform, plane);

    let mut sphere_a = Sphere::new(Color {r: 0.0, g: 0.0, b: 1.0});
    sphere_a.specular_strength = render_options.specular_strengths[1];
    let sphere_a_transform = &lookat_transform * &render_options.transformations[1];
    let transformed_sphere_a = TransformedSurface::new(sphere_a_transform, sphere_a);

    let sphere_b = Sphere::new(Color {r: 1.0, g: 0.0, b: 0.0});
    let sphere_b_transform = &lookat_transform * &render_options.transformations[2];
    let transformed_sphere_b = TransformedSurface::new(sphere_b_transform, sphere_b);

    let cone = Cone {
        apex: Point {x: 0.0, y: 0.0, z: 0.0},
        half_angle: 0.5,
        height: 0.5,
        color: Color {r: 0.0, g: 1.0, b: 0.0},
        specular_strength: render_options.specular_strengths[3],
    };
    let cone_transform = &lookat_transform * &render_options.transformations[3];
    let transformed_cone = TransformedSurface::new(cone_transform, cone);

    let light_sphere = Sphere::new(lights[0].color.clone());
    let light_transform = &lookat_transform * &render_options.transformations[4];
    let light_transformed = TransformedSurface::new(light_transform, light_sphere);

    Scene {
        objects: vec![
            Box::new(transformed_plane),
            Box::new(transformed_sphere_a),
            Box::new(transformed_sphere_b),
            Box::new(transformed_cone),
        ],
        camera: Camera::from_z_position(-1.0, render_options.fov, render_options.projection_type, WIDTH, HEIGHT),
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
            spheres_fly_radius: 3.0,
            spheres_fly_speed: 0.3,
            specular_strengths: [0.0, 0.0, 0.0, 0.0, 0.0],
            fov: std::f32::consts::PI * 0.5,
            camera_opts: CameraOptions {
                yaw: -0.5 * std::f32::consts::PI,
                pitch: 0.0,
                position: Vec3 {x: 0.0, y: 0.0, z: -5.0},
            },
            transformations: [
                AffineMat3::identity(),
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 0.25,
                    translation: Vec3::new(0.0, 0.0, 0.0),
                },
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 0.25,
                    translation: Vec3::new(0.0, 0.0, 0.0),
                },
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 1.0,
                    translation: Vec3::new(0.0, 0.0, 0.0),
                },
                AffineMat3 {
                    transform_mat: Mat3::identity(),
                    translation: Vec3::new(0.0, 5.0, 0.0),
                }
            ],
        }
    }

    fn update_transformations_on_time(&mut self, time: f32) {
        self.transformations[1].translation.x = (time * self.spheres_fly_speed).sin() * self.spheres_fly_radius;
        self.transformations[1].translation.z = (time * self.spheres_fly_speed).cos() * self.spheres_fly_radius;
        self.transformations[2].translation.x = -(time * self.spheres_fly_speed).sin() * self.spheres_fly_radius;
        self.transformations[2].translation.z = -(time * self.spheres_fly_speed).cos() * self.spheres_fly_radius;
    }
}