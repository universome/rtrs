use std::time::{Instant};

use rayon::prelude::*;
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};

use crate::scene::Scene;
use crate::camera::{Camera, ProjectionType};
use crate::surface::surface::{TransformedSurface, VisualData, Surface};
use crate::surface::quadrics::{Sphere, Plane, Cone};
use crate::surface::aabb::{AxisAlignedBox};
use crate::surface::mesh::{TriangleMesh};
use crate::basics::*;
use crate::matrix::{Mat3, AffineMat3};

// static WIDTH: u32 = 640;
// static HEIGHT: u32 = 480;
// static WIDTH: u32 = 800;
// static HEIGHT: u32 = 600;
// static WIDTH: u32 = 960;
// static HEIGHT: u32 = 720;
static WIDTH: u32 = 1280;
static HEIGHT: u32 = 960;


pub struct State {
    pub opts: RenderOptions,
    pub is_mouse_inited: bool,
    pub curr_mouse_x: f32,
    pub curr_mouse_y: f32,
    pub mouse_sensitivity: f32,
    pub move_speed: f32,
    pub mouse_is_in_window: bool,
    pub scroll_speed: f32,
    pub rotation_speed: f32,
    pub scale_speed: f32,
    pub selected_scene_idx: u32,
    pub simple_teapot: TriangleMesh,
    pub teapot: TriangleMesh,
    pub teacup: TriangleMesh,
    pub spoon: TriangleMesh,
}


impl State {
    pub fn setup_lights(render_options: &RenderOptions) -> Vec<Light> {
        // vec![Light {
        //     location: Point {x: -0.25, y: 10.0, z: -0.25},
        //     color: Color {r: 1.0, g: 1.0, b: 1.0},
        //     right: Vec3::new(0.5, 0.0, 0.0),
        //     top: Vec3::new(0.0, 0.0, 0.5),
        // }]
        let lookat_transform = render_options.camera_opts.compute_lookat();
        vec![Light {
            location: &lookat_transform * &Point {x: -0.1, y: 10.0, z: -0.1},
            color: Color {r: 1.0, g: 1.0, b: 1.0},
            right: &lookat_transform * &Vec3::new(0.2, 0.0, 0.0),
            top: &lookat_transform * &Vec3::new(0.0, 0.0, 0.2),
        }]
    }

    pub fn setup_plane(render_options: &RenderOptions) -> Box<dyn Surface> {
        let lookat_transform = render_options.camera_opts.compute_lookat();
        let plane = Plane::from_y(-1.4, Color {r: 0.5, g: 0.5, b: 0.5});
        let plane_transform = &lookat_transform * &render_options.object_transformations[0];
        let transformed_plane = TransformedSurface::new(plane_transform, plane);

        Box::new(transformed_plane)
    }

    pub fn setup_mesh_scene_objects(&self, render_options: &RenderOptions) -> Vec<Box<dyn Surface>> {
        let lookat_transform = render_options.camera_opts.compute_lookat();
        let mut teapot = self.teapot.clone();
        let mut teacup = self.teacup.clone();
        let mut spoon = self.spoon.clone();

        teapot.vis.reflection_glossiness = render_options.reflection_glossiness;
        teacup.vis.reflection_glossiness = render_options.reflection_glossiness;
        spoon.vis.reflection_glossiness = render_options.reflection_glossiness;

        let teapot_transform = &lookat_transform * &render_options.teaset_transformations[0];
        let transformed_teapot = TransformedSurface::new(teapot_transform, teapot);

        let teacup_transform = &lookat_transform * &render_options.teaset_transformations[1];
        let transformed_teacup = TransformedSurface::new(teacup_transform, teacup);

        let spoon_transform = &lookat_transform * &render_options.teaset_transformations[2];
        let transformed_spoon = TransformedSurface::new(spoon_transform, spoon);

        vec![
            Box::new(transformed_teapot),
            Box::new(transformed_teacup),
            Box::new(transformed_spoon),
        ]
    }

    pub fn setup_simple_mesh_scene_objects(&self, render_options: &RenderOptions) -> Vec<Box<dyn Surface>> {
        let lookat_transform = render_options.camera_opts.compute_lookat();
        let mut simple_teapot = self.simple_teapot.clone();
        simple_teapot.vis.reflection_glossiness = render_options.reflection_glossiness;
        let mesh_transform = &lookat_transform * &render_options.simple_teapot_transformation;
        let transformed_mesh = TransformedSurface::new(mesh_transform, simple_teapot);

        vec![Box::new(transformed_mesh)]
    }

    pub fn setup_simple_scene_objects(render_options: &RenderOptions) -> Vec<Box<dyn Surface>> {
        let lookat_transform = render_options.camera_opts.compute_lookat();
        let mut sphere_a = Sphere::new(VisualData::from_color(&Color {r: 0.0, g: 0.0, b: 1.0}));
        sphere_a.vis.specular_strength = render_options.specular_strengths[1];
        let sphere_a_transform = &lookat_transform * &render_options.object_transformations[1];
        let transformed_sphere_a = TransformedSurface::new(sphere_a_transform, sphere_a);

        let sphere_b = Sphere::new(VisualData {
            color: Color {r: 1.0, g: 0.0, b: 0.0},
            specular_strength: 0.5,
            reflection_strength: 0.5,
            reflection_glossiness: render_options.reflection_glossiness,
        });
        let sphere_b_transform = &lookat_transform * &render_options.object_transformations[2];
        let transformed_sphere_b = TransformedSurface::new(sphere_b_transform, sphere_b);

        vec![Box::new(transformed_sphere_a), Box::new(transformed_sphere_b)]
    }

    pub fn compute_scene(&self) -> Scene {
        let objects = (match self.selected_scene_idx {
            0 => State::setup_simple_scene_objects(&self.opts),
            1 => self.setup_simple_mesh_scene_objects(&self.opts),
            2 => self.setup_mesh_scene_objects(&self.opts),
            _ => panic!("Wrong scene ID has been selected!")
        });
        let lights = State::setup_lights(&self.opts);
        let mut scene_objects = vec![State::setup_plane(&self.opts)];
        scene_objects.extend(objects);

        Scene {
            objects: scene_objects,
            camera: Camera::from_z_position(-1.0, self.opts.fov, self.opts.projection_type, WIDTH, HEIGHT),
            background_color: Color {r: 0.204, g: 0.596, b: 0.86},
            lights: lights,
            ambient_strength: 0.7,
            diffuse_strength: 0.5,
        }
    }
}


#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub projection_type: ProjectionType,
    pub number_of_lights: u32,
    pub camera_opts: CameraOptions,
    pub selected_pixel: Option<(u32, u32)>,
    pub selected_object_idx: Option<usize>,
    pub object_transformations: [AffineMat3; 3],
    pub simple_teapot_transformation: AffineMat3,
    pub teaset_transformations: [AffineMat3; 3],
    pub specular_strengths: [f32; 5],
    pub spheres_fly_radius: f32,
    pub spheres_fly_speed: f32,
    pub fov: f32,
    pub ray_opts: RayOptions,
    pub reflection_glossiness: f32,
    pub use_soft_shadows: bool,
    pub use_supersampling: bool,
}


#[derive(Debug, Clone)]
pub struct CameraOptions {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vec3,
}


impl CameraOptions {
    fn compute_lookat(&self) -> AffineMat3 {
        AffineMat3::create_look_at(&self.position, self.yaw, self.pitch)
    }
}


pub fn launch() {
    nannou::app(init_nannou).event(update_on_event).run();
}


fn init_nannou(app: &App) -> State {
    app
        .new_window()
        .title("CS248 Computer Graphics")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    init_state()
}


fn update_on_event(app: &App, state: &mut State, event: Event) {
    if state.selected_scene_idx == 0 {
        state.opts.update_transformations_on_time(app.time);
    }

    process_pressed_keys(app, state);
    process_mouse_events(app, state, event);
    // process_mouse_move(app, state);
}


fn process_key_released_event(app: &App, state: &mut State, key: Key) {
    match key {
        Key::F => {
            state.opts.ray_opts.mesh_normal_type = match state.opts.ray_opts.mesh_normal_type {
                MeshNormalType::Provided => MeshNormalType::Precomputed,
                MeshNormalType::Precomputed => MeshNormalType::Face,
                MeshNormalType::Face => MeshNormalType::Provided,
            };
            println!("Set bvh_display_level to {:?}", state.opts.ray_opts.mesh_normal_type);
        },
        Key::G => {
            state.opts.ray_opts.mesh_normal_type = match state.opts.ray_opts.mesh_normal_type {
                MeshNormalType::Precomputed => MeshNormalType::Provided,
                MeshNormalType::Face => MeshNormalType::Precomputed,
                MeshNormalType::Provided => MeshNormalType::Face,
            };
            println!("Set bvh_display_level to {:?}", state.opts.ray_opts.mesh_normal_type);
        },
        Key::B => {
            state.opts.ray_opts.bv_type = match state.opts.ray_opts.bv_type {
                BVType::BBox => BVType::Sphere,
                BVType::Sphere => BVType::None,
                BVType::None => BVType::BBox,
            };
            println!("Set bv_type to {:?}", state.opts.ray_opts.bv_type);
        },
        Key::Key1 => state.selected_scene_idx = 0,
        Key::Key2 => state.selected_scene_idx = 1,
        Key::Key3 => state.selected_scene_idx = 2,
        Key::Up => {
            state.opts.ray_opts.bvh_display_level += 1;
            println!("Set bvh_display_level to {}", state.opts.ray_opts.bvh_display_level);
        },
        Key::Down => {
            state.opts.ray_opts.bvh_display_level -= 1;
            println!("Set bvh_display_level to {}", state.opts.ray_opts.bvh_display_level);
        },
        Key::I => {
            state.opts.use_supersampling = !state.opts.use_supersampling;
            println!("Set use_supersampling to {}", state.opts.use_supersampling);
        },
        Key::O => {
            state.opts.use_soft_shadows = !state.opts.use_soft_shadows;
            println!("Set use_soft_shadows to {}", state.opts.use_soft_shadows);
        },
        Key::P => {
            state.opts.reflection_glossiness = if state.opts.reflection_glossiness == 0.0 {0.2} else {0.0};
            println!("Set reflection_glossiness to {}", state.opts.reflection_glossiness);
        },
        Key::Q => *state = init_state(),
        _ => {},
    }
}


fn process_pressed_keys(app: &App, state: &mut State) {
    let camera_transformation = AffineMat3::create_look_at(
        &state.opts.camera_opts.position,
        state.opts.camera_opts.yaw,
        state.opts.camera_opts.pitch,
    );

    if app.keys.down.contains(&Key::W) {
        state.opts.camera_opts.position = &state.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * -state.move_speed);
    }

    if app.keys.down.contains(&Key::S) {
        state.opts.camera_opts.position = &state.opts.camera_opts.position + &(&camera_transformation.transform_mat[2] * state.move_speed);
    }

    if app.keys.down.contains(&Key::D) {
        state.opts.camera_opts.position = &state.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * state.move_speed);
    }

    if app.keys.down.contains(&Key::A) {
        state.opts.camera_opts.position = &state.opts.camera_opts.position + &(&camera_transformation.transform_mat[0] * -state.move_speed);
    }

    // if app.keys.down.contains(&Key::L) {
    //     state.opts.selected_object_idx = Some(4); // Selecting the light
    // }

    // if let Some(idx) = state.opts.selected_object_idx {
    //     let mut transformation = None;

    //     if app.keys.down.contains(&Key::Key1) {
    //         if app.keys.down.contains(&Key::Up) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0 + state.scale_speed, 1.0, 1.0)));
    //         } else if app.keys.down.contains(&Key::Down) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0 - state.scale_speed, 1.0, 1.0)));
    //         }
    //     } else if app.keys.down.contains(&Key::Key2) {
    //         if app.keys.down.contains(&Key::Up) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0 + state.scale_speed, 1.0)));
    //         } else if app.keys.down.contains(&Key::Down) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0 - state.scale_speed, 1.0)));
    //         }
    //     } else if app.keys.down.contains(&Key::Key3) {
    //         if app.keys.down.contains(&Key::Up) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0, 1.0 + state.scale_speed)));
    //         } else if app.keys.down.contains(&Key::Down) {
    //             transformation = Some(AffineMat3::scale(Vec3::new(1.0, 1.0, 1.0 - state.scale_speed)));
    //         }
    //     } else if app.keys.down.contains(&Key::Up) {
    //         transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[1] * state.move_speed));
    //     } else if app.keys.down.contains(&Key::Down) {
    //         transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[1] * -state.move_speed));
    //     } else if app.keys.down.contains(&Key::Right) {
    //         transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[0] * state.move_speed));
    //     } else if app.keys.down.contains(&Key::Left) {
    //         transformation = Some(AffineMat3::translation(&camera_transformation.transform_mat[0] * -state.move_speed));
    //     } else if app.keys.down.contains(&Key::I) {
    //         transformation = Some(AffineMat3::rotation(state.rotation_speed, &Vec3::new(1.0, 0.0, 0.0)));
    //     } else if app.keys.down.contains(&Key::O) {
    //         transformation = Some(AffineMat3::rotation(state.rotation_speed, &Vec3::new(0.0, 1.0, 0.0)));
    //     } else if app.keys.down.contains(&Key::P) {
    //         transformation = Some(AffineMat3::rotation(state.rotation_speed, &Vec3::new(0.0, 0.0, 1.0)));
    //     }

    //     if transformation.is_some() {
    //         state.opts.transformations[idx] = &state.opts.transformations[idx] * &transformation.unwrap();
    //     }
    // }
}


fn process_mouse_events(app: &App, state: &mut State, event: Event) {
    match event {
        Event::WindowEvent {id: _, simple: window_event } => {
            if window_event.is_none() {
                return;
            }

            match window_event.unwrap() {
                MouseEntered => {
                    println!("Mouse entered!");
                    state.mouse_is_in_window = true;
                    state.is_mouse_inited = false;
                },
                MouseExited => {
                    state.mouse_is_in_window = false;
                    state.is_mouse_inited = false;
                    state.opts.selected_pixel = None;
                    state.opts.specular_strengths = [0.0, 0.0, 0.0, 0.0, 1.0];
                },
                // MousePressed(button) => {
                    // if button != MouseButton::Left {
                    //     return;
                    // }

                    // let i = (state.curr_mouse_x + (WIDTH as f32) / 2.0) as u32;
                    // let j = (state.curr_mouse_y + (HEIGHT as f32) / 2.0) as u32;

                    // state.scene.compute_pixel(i, j, true);

                    // if let Some(obj_idx) = state.scene.get_object_idx_at_pixel(i, j) {
                    //     state.opts.selected_object_idx = Some(obj_idx);
                    //     state.opts.specular_strengths[obj_idx] = 0.7;
                    // } else {
                    //     state.opts.selected_object_idx = None;
                    //     state.opts.specular_strengths = [0.0, 0.0, 0.0, 0.0, 1.0];
                    // }

                    // dbg!(&state.opts.camera_opts.position);
                // },
                KeyReleased(key) => process_key_released_event(app, state, key),
                MouseWheel(scroll_delta, _) => {
                    match scroll_delta {
                        MouseScrollDelta::PixelDelta(position) => {
                            state.opts.fov += (position.y as f32) * state.scroll_speed;
                            state.opts.fov = state.opts.fov
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


fn process_mouse_move(app: &App, state: &mut State) {
    if !state.mouse_is_in_window {
        return;
    }

    if !state.is_mouse_inited {
        state.curr_mouse_x = app.mouse.x;
        state.curr_mouse_y = app.mouse.y;
        state.is_mouse_inited = true;
    }

    let offset_x = (app.mouse.x - state.curr_mouse_x) * state.mouse_sensitivity;
    let offset_y = (state.curr_mouse_y - app.mouse.y) * state.mouse_sensitivity;

    state.curr_mouse_x = app.mouse.x;
    state.curr_mouse_y = app.mouse.y;
    state.opts.camera_opts.yaw += offset_x;
    state.opts.camera_opts.pitch += offset_y;

    state.opts.camera_opts.pitch = state.opts.camera_opts.pitch
        .min(0.5 * std::f32::consts::PI - 0.001)
        .max(-0.5 * std::f32::consts::PI + 0.001);

    // (*app.main_window()).set_cursor_position_points(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    // state.curr_mouse_x = app.mouse.x;
    // state.curr_mouse_y = app.mouse.y;
}


fn view(app: &App, state: &State, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let start = Instant::now();
    let img = render_state(state);
    let duration = start.elapsed();
    println!("Rending took time: {:?}", duration);

    img.save("image.png").unwrap();

    draw.texture(&wgpu::Texture::from_image(app, &img));
    draw.to_frame(app, &frame).unwrap();
}


fn init_state() -> State {
    println!("Building state..");

    let render_options = RenderOptions::defaults();
    let mesh_vis = VisualData {
        color: Color {r: 0.769, g: 0.792, b: 0.808},
        specular_strength: 0.2,
        reflection_strength: 0.2,
        reflection_glossiness: 0.0,
    };

    State {
        selected_scene_idx: 0,
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
        simple_teapot: TriangleMesh::from_obj("resources/teapot.obj", mesh_vis.clone()),
        teapot: TriangleMesh::from_obj("resources/newell_teaset/teapot.obj", mesh_vis.clone()),
        teacup: TriangleMesh::from_obj("resources/newell_teaset/teacup.obj", mesh_vis.clone()),
        spoon: TriangleMesh::from_obj("resources/newell_teaset/spoon.obj", mesh_vis.clone()),
    }
}


pub fn render_state(state: &State) -> DynamicImage {
    let scene = state.compute_scene();
    let pixels = iproduct!(0..HEIGHT, 0..WIDTH)
        .collect::<Vec<(u32, u32)>>()
        .par_iter()
        .map(|p: &(u32, u32)| -> Color {
            scene.compute_pixel(p.1, HEIGHT - p.0, &state.opts)
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


impl RenderOptions {
    fn defaults() -> Self {
        RenderOptions {
            use_soft_shadows: false,
            use_supersampling: false,
            reflection_glossiness: 0.0,
            ray_opts: RayOptions::from_depth(0),
            projection_type: ProjectionType::Perspective,
            number_of_lights: 1,
            selected_pixel: None,
            selected_object_idx: None,
            spheres_fly_radius: 2.0,
            spheres_fly_speed: 0.3,
            specular_strengths: [0.0, 0.0, 0.0, 0.0, 0.0],
            fov: std::f32::consts::PI * 0.5,
            camera_opts: CameraOptions {
                yaw: -0.5 * std::f32::consts::PI,
                pitch: 0.0,
                position: Vec3 {x: 0.0, y: 0.0, z: -7.0},
            },
            simple_teapot_transformation: AffineMat3 {
                transform_mat: &Mat3::identity() * 0.1,
                translation: Vec3::new(0.0, 0.0, 0.0),
            },
            teaset_transformations: [
                AffineMat3 {
                    transform_mat: &Mat3::rotation(-std::f32::consts::PI * 0.5, &Vec3::new(0.0, 1.0, 0.0)) * &(&Mat3::identity() * 0.5),
                    translation: Vec3::new(-1.5, -1.4, 0.0),
                },
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 0.5,
                    translation: Vec3::new(0.5, -1.4, 0.0),
                },
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 2.0,
                    translation: Vec3::new(2.5, -1.4, 0.0),
                }
            ],
            object_transformations: [
                AffineMat3::identity(),
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 0.5,
                    translation: Vec3::new(-1.0, 0.0, 0.0),
                },
                AffineMat3 {
                    transform_mat: &Mat3::identity() * 0.5,
                    translation: Vec3::new(1.0, 0.0, 0.0),
                }
            ],
        }
    }

    fn update_transformations_on_time(&mut self, time: f32) {
        self.object_transformations[1].translation.x = (time * self.spheres_fly_speed).sin() * self.spheres_fly_radius;
        self.object_transformations[1].translation.z = (time * self.spheres_fly_speed).cos() * self.spheres_fly_radius;
        self.object_transformations[2].translation.x = -(time * self.spheres_fly_speed).sin() * self.spheres_fly_radius;
        self.object_transformations[2].translation.z = -(time * self.spheres_fly_speed).cos() * self.spheres_fly_radius;
    }
}
