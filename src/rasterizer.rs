use std::time::{Instant, Duration};
use std::f32::consts::{PI};
use std::cmp;
use std::env;

// use nalgebra::{Matrix4, Vector3, Point3};
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};
use tobj::{Model};

use crate::matrix::*;
use crate::basics::*;

// const WIDTH: usize = 640;
// const HEIGHT: usize = 480;
const WIDTH: usize = 1280;
const HEIGHT: usize = 960;

#[derive(Debug, Clone)]
struct State {
    model: Model,
    camera: Camera,
    curr_mouse_x: f32,
    curr_mouse_y: f32,
    object_to_world: AffineMat3,
    arcball_enabled: bool,
    light_position: Point,
    is_gouraud_shading: bool,
    is_antialiasing: bool,
    specular_lighting_enabled: bool,
    tex_enabled: bool,
}

#[derive(Debug, Clone)]
struct Camera {
    pub distance: f32, // Camera distance
    pub fov: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
}

#[derive(Debug, Clone)]
struct ViewingPlane {
    pub z: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}


impl Camera {
    fn compute_view_matrix(&self) -> AffineMat3 {
        let eye = Point::new(0.0, 0.0, -self.distance);
        let look_at = Point::new(0.0, 0.0, -self.distance - 1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        AffineMat3::new_view_matrix(&eye, &look_at, &up)
    }

    fn compute_arcball_vector_for_xy(x: f32, y: f32) -> Vec3 {
        // Computing x, y, in [-1, 1] coords
        // Nannou gives (x, y) coords in [-w/2, w/2] and [-h/2, h/2] formats
        let mut result = Vec3::new(2.0 * x / WIDTH as f32, 2.0 * y / HEIGHT as f32, 0.0);

        // Now we pretend that there is a ball which touches (0, 0) point
        // and which center is located in the object center
        // We want to project the point back to this sphere
        let curr_norm = (result.x.powi(2) + result.y.powi(2)).sqrt();
        if curr_norm < 1.0 {
            result.z = (1.0 - curr_norm * curr_norm).sqrt();
        } else {
            result = result.normalize();
        }

        result
    }

    pub fn compute_viewing_plane(&self, frame_width: usize, frame_height: usize) -> ViewingPlane {
        let y_half = (self.fov * 0.5).tanh();
        let x_half = y_half * (frame_width as f32) / (frame_height as f32);

        ViewingPlane {
            z: self.distance + self.near_clipping_plane / (self.fov * 0.5).tanh(),
            x_min: -x_half,
            x_max: x_half,
            y_min: -y_half,
            y_max: y_half,
        }
    }
}


pub fn launch() {
    nannou::app(init_app).event(update_on_event).run();
}


fn update_on_event(app: &App, state: &mut State, event: Event) {
    match event {
        Event::WindowEvent {id: _, simple: window_event } => {
            if window_event.is_none() {
                return;
            }

            match window_event.unwrap() {
                MousePressed(button) => {
                    if button != MouseButton::Left {
                        return;
                    }

                    state.arcball_enabled = true;
                    state.curr_mouse_x = app.mouse.x;
                    state.curr_mouse_y = app.mouse.y;
                },
                MouseReleased(button) => {
                    if button != MouseButton::Left {
                        return;
                    }

                    state.arcball_enabled = false;
                },
                KeyPressed(key) => {
                    if key == Key::L {
                        state.is_gouraud_shading = !state.is_gouraud_shading;
                    }

                    if key == Key::A {
                        state.is_antialiasing = !state.is_antialiasing;
                    }

                    if key == Key::S {
                        state.specular_lighting_enabled = !state.specular_lighting_enabled;
                    }

                    if key == Key::T {
                        state.tex_enabled = !state.tex_enabled;
                    }
                },
                _ => {}
            }
        },
        _ => {},
    }

    if !state.arcball_enabled {
        return;
    }

    if state.curr_mouse_x == app.mouse.x && state.curr_mouse_y == app.mouse.y {
        return;
    }

    let prev_arcball_vec = Camera::compute_arcball_vector_for_xy(state.curr_mouse_x, state.curr_mouse_y);
    let curr_arcball_vec = Camera::compute_arcball_vector_for_xy(app.mouse.x, app.mouse.y);
    let angle = 2.0 * prev_arcball_vec.dot_product(&curr_arcball_vec).min(1.0).acos();
    let world_to_camera = state.camera.compute_view_matrix();
    let camera_to_world = &world_to_camera.compute_inverse();
    let axis_camera = &prev_arcball_vec.cross_product(&curr_arcball_vec);
    let axis_world = (camera_to_world * axis_camera).normalize();
    let rotation = AffineMat3::rotation(angle, &axis_world);

    // state.object_rotation = &rotation * &state.object_rotation;
    state.object_to_world = &rotation * &state.object_to_world;
    state.curr_mouse_x = app.mouse.x;
    state.curr_mouse_y = app.mouse.y;
}


fn init_app(app: &App) -> State {
    let obj_file = env::args()
        .skip(1)
        .next()
        .expect("A .obj file to print is required");
    let (models, _) = tobj::load_obj(&obj_file, true).unwrap();

    app
        .new_window()
        .title("CS248 Computer Graphics")
        .size(WIDTH as u32, HEIGHT as u32)
        .view(render_and_display)
        .build()
        .unwrap();

    let mut state = init_state(models[0].clone());

    (*app.main_window()).set_cursor_position_points(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    state.curr_mouse_x = app.mouse.x;
    state.curr_mouse_y = app.mouse.y;

    state
}


fn render_and_display(app: &App, state: &State, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let img = render_state(state);
    let texture = wgpu::Texture::from_image(app, &img);

    draw.texture(&texture);
    draw.to_frame(app, &frame).unwrap();
}


fn render_state(state: &State) -> DynamicImage{
    let model = &state.model;
    let num_triangles = model.mesh.num_face_indices.len();
    let tex = &model.mesh.texcoords;
    let world_to_camera = state.camera.compute_view_matrix();
    let object_to_camera = &world_to_camera * &state.object_to_world;
    let light_pos_camera = &world_to_camera * &state.light_position;
    let frame_width: usize = if state.is_antialiasing {WIDTH * 2} else {WIDTH};
    let frame_height: usize = if state.is_antialiasing {HEIGHT * 2} else {HEIGHT};
    let viewing_plane = state.camera.compute_viewing_plane(frame_width, frame_height);
    let bg_color = Color::new(236.0 / 255.0, 240.0 / 255.0, 241.0 / 255.0);
    let mut frame_buffer = vec![bg_color; frame_width * frame_height];
    let mut z_buffer = vec![state.camera.far_clipping_plane; frame_width * frame_height];

    let start = Instant::now();

    for i in 0..(num_triangles as usize) {
        let idx_1 = model.mesh.indices[i * 3 + 0] as usize;
        let idx_2 = model.mesh.indices[i * 3 + 1] as usize;
        let idx_3 = model.mesh.indices[i * 3 + 2] as usize;

        let v0 = Point::new(model.mesh.positions[idx_1 * 3 + 0], model.mesh.positions[idx_1 * 3 + 1], model.mesh.positions[idx_1 * 3 + 2]);
        let v1 = Point::new(model.mesh.positions[idx_2 * 3 + 0], model.mesh.positions[idx_2 * 3 + 1], model.mesh.positions[idx_2 * 3 + 2]);
        let v2 = Point::new(model.mesh.positions[idx_3 * 3 + 0], model.mesh.positions[idx_3 * 3 + 1], model.mesh.positions[idx_3 * 3 + 2]);

        // Convert the vertices of the triangle to raster space
        let mut v0_raster = convert_to_raster(&v0, &object_to_camera, &state.camera, &viewing_plane, frame_width, frame_height);
        let mut v1_raster = convert_to_raster(&v1, &object_to_camera, &state.camera, &viewing_plane, frame_width, frame_height);
        let mut v2_raster = convert_to_raster(&v2, &object_to_camera, &state.camera, &viewing_plane, frame_width, frame_height);

        // Precompute reciprocal of vertex z-coordinate
        v0_raster.z = 1.0 / v0_raster.z;
        v1_raster.z = 1.0 / v1_raster.z;
        v2_raster.z = 1.0 / v2_raster.z;

        // Gouraud shading coloring
        // TODO: the best option would be to compute the normal and v_cam inside the first run...
        let v0_camera = &object_to_camera * &v0;
        let v1_camera = &object_to_camera * &v1;
        let v2_camera = &object_to_camera * &v2;
        let light_dirs = (
            (&light_pos_camera - &v0_camera).normalize(),
            (&light_pos_camera - &v1_camera).normalize(),
            (&light_pos_camera - &v2_camera).normalize(),
        );
        let face_normal_camera = (&((&v1_camera - &v0_camera).cross_product(&(&v2_camera - &v0_camera)))).normalize();

        // Making backface culling
        let v0_view_direction = (-&Vec3::new(v0_camera.x, v0_camera.y, v0_camera.z)).normalize();
        if v0_view_direction.dot_product(&face_normal_camera) < 0.0 {
            continue;
        }

        let colors_gouraud = (
            face_normal_camera.dot_product(&light_dirs.0),
            face_normal_camera.dot_product(&light_dirs.1),
            face_normal_camera.dot_product(&light_dirs.2),
        );

        let mut gouraud_speculars = (0.0, 0.0, 0.0);
        if state.specular_lighting_enabled {
            gouraud_speculars = (
                compute_specular(&face_normal_camera, &(-&Vec3::new(v0_camera.x, v0_camera.y, v0_camera.z)).normalize(), &light_dirs.0),
                compute_specular(&face_normal_camera, &(-&Vec3::new(v1_camera.x, v1_camera.y, v1_camera.z)).normalize(), &light_dirs.1),
                compute_specular(&face_normal_camera, &(-&Vec3::new(v2_camera.x, v2_camera.y, v2_camera.z)).normalize(), &light_dirs.2),
            );
        }

        let (mut normal_v0_camera, mut normal_v1_camera, mut normal_v2_camera) = (Vec3::zero(), Vec3::zero(), Vec3::zero());
        if !state.is_gouraud_shading {
            let normal_v0 = Vec3::new(model.mesh.normals[idx_1 * 3 + 0], model.mesh.normals[idx_1 * 3 + 1], model.mesh.normals[idx_1 * 3 + 2]);
            let normal_v1 = Vec3::new(model.mesh.normals[idx_2 * 3 + 0], model.mesh.normals[idx_2 * 3 + 1], model.mesh.normals[idx_2 * 3 + 2]);
            let normal_v2 = Vec3::new(model.mesh.normals[idx_3 * 3 + 0], model.mesh.normals[idx_3 * 3 + 1], model.mesh.normals[idx_3 * 3 + 2]);

            normal_v0_camera = &object_to_camera * &normal_v0;
            normal_v1_camera = &object_to_camera * &normal_v1;
            normal_v2_camera = &object_to_camera * &normal_v2;
        }

        // Prepare vertex attributes. Divde them by their vertex z-coordinate
        // (though we use a multiplication here because v.z = 1 / v.z)
        let mut st0 = (0.0, 0.0);
        let mut st1 = (0.0, 0.0);
        let mut st2 = (0.0, 0.0);

        if !tex.is_empty() && state.tex_enabled {
            st0 = (tex[idx_1 * 2] * v0_raster.z, tex[idx_1 * 2 + 1] * v0_raster.z);
            st1 = (tex[idx_2 * 2] * v1_raster.z, tex[idx_2 * 2 + 1] * v1_raster.z);
            st2 = (tex[idx_3 * 2] * v2_raster.z, tex[idx_3 * 2 + 1] * v2_raster.z);
        }

        let x_min = min_of_three(v0_raster.x, v1_raster.x, v2_raster.x);
        let y_min = min_of_three(v0_raster.y, v1_raster.y, v2_raster.y);
        let x_max = max_of_three(v0_raster.x, v1_raster.x, v2_raster.x);
        let y_max = max_of_three(v0_raster.y, v1_raster.y, v2_raster.y);

        // the triangle is out of screen
        if x_min > (frame_width - 1) as f32 || x_max < 0.0 || y_min > (frame_height - 1) as f32 || y_max < 0.0 {
            continue;
        }

        // be careful x_min/x_max/y_min/y_max can be negative. Don't cast to uint32_t
        let x0 = cmp::max(0, x_min.floor() as i32) as usize;
        let x1 = cmp::min(frame_width as i32 - 1, x_max.floor() as i32) as usize;
        let y0 = cmp::max(0, y_min.floor() as i32) as usize;
        let y1 = cmp::min(frame_height as i32 - 1, y_max.floor() as i32) as usize;

        let area = edge_function(&v0_raster, &v1_raster, &v2_raster);

        for y in y0..(y1 + 1) {
            for x in x0..(x1 + 1) {
                let pixel_pos = Point::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let bar_coords = (
                    edge_function(&v1_raster, &v2_raster, &pixel_pos) / area,
                    edge_function(&v2_raster, &v0_raster, &pixel_pos) / area,
                    edge_function(&v0_raster, &v1_raster, &pixel_pos) / area,
                );

                if bar_coords.0 >= 0.0 && bar_coords.1 >= 0.0 && bar_coords.2 >= 0.0 {
                    let depth = 1.0 / (v0_raster.z * bar_coords.0 + v1_raster.z * bar_coords.1 + v2_raster.z * bar_coords.2);

                    if depth < z_buffer[y * frame_width + x] {
                        z_buffer[y * frame_width + x] = depth;

                        let mut color = 0.1; // Ambient strength

                        if state.is_gouraud_shading {
                            let diffuse_strength = 0.7 * colors_gouraud.0 * bar_coords.0 + colors_gouraud.1 * bar_coords.1 + colors_gouraud.2 * bar_coords.2;
                            color += diffuse_strength;

                            if state.specular_lighting_enabled {
                                color += gouraud_speculars.0 * bar_coords.0 + gouraud_speculars.1 * bar_coords.1 + gouraud_speculars.2 * bar_coords.2;
                            }
                        } else {
                            let px = (v0_camera.x / -v0_camera.z) * bar_coords.0 + (v1_camera.x / -v1_camera.z) * bar_coords.1 + (v2_camera.x / -v2_camera.z) * bar_coords.2;
                            let py = (v0_camera.y / -v0_camera.z) * bar_coords.0 + (v1_camera.y / -v1_camera.z) * bar_coords.1 + (v2_camera.y / -v2_camera.z) * bar_coords.2;
                            let pos_camera = Point::new(px * depth, py * depth, -depth); // fragmet position is in the camera space
                            let light_dir = (&light_pos_camera - &pos_camera).normalize();
                            // let point_normal_camera = normal_v0_camera * bar_coords.0 + normal_v1_camera * bar_coords.1 + normal_v2_camera * bar_coords.2;
                            let point_normal_camera = (&normal_v0_camera * bar_coords.0  + &normal_v1_camera * bar_coords.1  + &normal_v2_camera * bar_coords.2).normalize();
                            let diffuse_strength = point_normal_camera.dot_product(&light_dir);
                            color += diffuse_strength;

                            if state.specular_lighting_enabled {
                                let view_direction = (-&Vec3::new(pos_camera.x, pos_camera.y, pos_camera.z)).normalize();

                                color += compute_specular(&point_normal_camera, &view_direction, &light_dir);
                            }
                        }

                        if !tex.is_empty() && state.tex_enabled {
                            let tex_coords = (
                                (st0.0 * bar_coords.0 + st1.0 * bar_coords.1 + st2.0 * bar_coords.2) * depth,
                                (st0.1 * bar_coords.0 + st1.1 * bar_coords.1 + st2.1 * bar_coords.2) * depth,
                            );

                            color += compute_stripe_color(tex_coords.0, tex_coords.1);
                        }

                        frame_buffer[y * frame_width + x] = Color::new(color, color, color);
                    }
                }
            }
        }
    }

    let duration = start.elapsed();
    println!("Rasterizer done! Took time: {} ms", duration.as_millis());

    let mut img = RgbImage::new(WIDTH as u32, HEIGHT as u32);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut color = Color::zero();
            if state.is_antialiasing {
                // Mean filtering
                color = color.add_no_clamp(&frame_buffer[WIDTH * 2 * (y * 2 + 0) + x * 2 + 0]);
                color = color.add_no_clamp(&frame_buffer[WIDTH * 2 * (y * 2 + 0) + x * 2 + 1]);
                color = color.add_no_clamp(&frame_buffer[WIDTH * 2 * (y * 2 + 1) + x * 2 + 0]);
                color = color.add_no_clamp(&frame_buffer[WIDTH * 2 * (y * 2 + 1) + x * 2 + 1]);
                color = &color * 0.25;
            } else {
                color = frame_buffer[WIDTH * y + x];
            };

            img.put_pixel(x as u32, y as u32, color.into());
        }
    }

    let img = DynamicImage::ImageRgb8(img);
    // img.save("image.tga").unwrap();

    img
}


fn init_state(model: Model) -> State {
    println!("Building model!");
    let distance = -2.0;

    let mut object_center = Point::zero();
    for i in 0..((model.mesh.positions.len() / 3) as usize) {
        object_center.x += model.mesh.positions[i * 3 + 0] * (1.0 / model.mesh.positions.len() as f32);
        object_center.y += model.mesh.positions[i * 3 + 1] * (1.0 / model.mesh.positions.len() as f32);
        object_center.z += model.mesh.positions[i * 3 + 2] * (1.0 / model.mesh.positions.len() as f32);
    }

    println!("Object center: {:?}", &object_center);
    println!("Number of vertices: {}", model.mesh.positions.len() / 3);
    println!("Number of normals: {}", model.mesh.normals.len() / 3);
    println!("Number of texcoords: {}", model.mesh.texcoords.len() / 2);

    State {
        model: model,
        object_to_world: AffineMat3::translation((&-&(&object_center * 2.5)).into()),
        camera: Camera {
            distance: distance,
            fov: PI * 0.5,
            near_clipping_plane: 1.0,
            far_clipping_plane: 1000.0,
        },
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        arcball_enabled: false,
        light_position: Point::new(0.0, 100.0, 0.0),
        is_gouraud_shading: true,
        is_antialiasing: false,
        specular_lighting_enabled: false,
        tex_enabled: false,
    }
}


fn convert_to_raster(
    vertex_obj: &Point, object_to_camera: &AffineMat3, camera: &Camera, viewing_plane: &ViewingPlane, frame_width: usize, frame_height: usize) -> Point {

    // To camera space
    let mut result = object_to_camera * vertex_obj;

    // To clip space
    // 1. Apply perspective
    result.x = camera.near_clipping_plane * result.x / -result.z;
    result.y = camera.near_clipping_plane * result.y / -result.z;
    // 2.  Convert to [-1, 1]
    result.x = (2.0 * result.x - (viewing_plane.x_max + viewing_plane.x_min)) / (viewing_plane.x_max - viewing_plane.x_min);
    result.y = (2.0 * result.y - (viewing_plane.y_max + viewing_plane.y_min)) / (viewing_plane.y_max - viewing_plane.y_min);

    // To screen space, i.e [0, w] and [0, h]
    result.x = (result.x + 1.0) * 0.5 * (frame_width as f32);
    result.y = (1.0 - result.y) * 0.5 * (frame_height as f32);
    result.z = -result.z;

    result
}


#[inline]
fn edge_function(u: &Point, v: &Point, point: &Point) -> f32 {
    // Given two vectors u, v, computes the edge function for the given point
    (point.x - u.x) * (v.y - u.y) - (point.y - u.y) * (v.x - u.x)
}


#[inline]
fn min_of_three(a: f32, b: f32, c: f32) -> f32 {
    if a < b {
        if a < c { a } else { c }
    } else {
        if b < c { b } else { c }
    }
}

#[inline]
fn max_of_three(a: f32, b: f32, c: f32) -> f32 {
    if a > b {
        if a > c { a } else { c }
    } else {
        if b > c { b } else { c }
    }
}

#[inline]
fn compute_specular(normal: &Vec3, view_dir: &Vec3, light_dir: &Vec3) -> f32 {
    let normal_dot_light = normal.dot_product(light_dir).max(0.0);
    let reflect_dir = &(&-light_dir + &(normal * (2.0 * normal_dot_light)));
    let reflect_dot_view = view_dir.dot_product(&reflect_dir).max(0.0);

    0.5 * reflect_dot_view.powi(32)
}

#[inline]
fn compute_stripe_color(_s: f32, t: f32) -> f32 {
    let stripes_fuzz = 0.1;
    let stripes_width = 0.5;
    let stripes_scale = 20.0;

    let scaled_t = (t * stripes_scale) % 1.0; // Split into 10 stripes
    let step_1 = (scaled_t / stripes_fuzz).min(1.0).max(0.0);
    let step_2 = ((scaled_t - stripes_width) / stripes_fuzz).min(1.0).max(0.0);
    let step_3 = step_1 * (1.0 - step_2);
    let step_4 = step_3 * step_3 * (3.0 - (2.0 * step_3));

    let back_color = 0.0;
    let stripe_color = 0.7;

    back_color * step_4 + (1.0 - step_4) * stripe_color
}
