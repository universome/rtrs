use std::time::{Instant, Duration};
use std::cmp;
use std::f32::consts::{PI};
use std::env;

use nalgebra::{Matrix4, Vector3, Point3};
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};
use tobj::{Model};

// use crate::mesh::{Triangle};
use crate::matrix::*;
use crate::basics::*;

const inch_to_mm: f32 = 25.4;
// const WIDTH: usize = 160;
// const HEIGHT: usize = 120;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
// const WIDTH: usize = 1280;
// const HEIGHT: usize = 960;

struct State {
    model: Model,
    camera: Camera,
    curr_mouse_x: f32,
    curr_mouse_y: f32,
    object_to_world: AffineMat3,
    arcball_enabled: bool,
    light_position: Point,
    is_gouraud_shading: bool,
}

struct Camera {
    pub distance: f32, // Determines the ball radius for arcball
    pub eye: Point,
    pub look_at: Point,
    pub up: Vec3,
}


impl Camera {
    fn compute_view_matrix(&self) -> AffineMat3 {
        AffineMat3::new_view_matrix(&self.eye, &self.look_at, &self.up)
    }

    fn compute_arcball_vector_for_xy(x: f32, y: f32) -> Vec3 {
        // Computing x, y, in [-1, 1] coords
        // Nannou gives (x, y) coords in [-w/2, w/2] and [-h/2, h/2] formats
        let mut result = Vec3::new(2.0 * x / WIDTH as f32, 2.0 * y / HEIGHT as f32, 0.0);

        // Now we pretend that there is a ball which touches (0, 0) point
        // and which center is located in the object center
        // We want to project the point back to this sphere
        let curr_norm = (result.x.powi(2) + result.y.powi(2)).sqrt();
        if (curr_norm < 1.0) {
            result.z = (1.0 - curr_norm * curr_norm).sqrt();
        } else {
            result = result.normalize();
        }

        result
    }
}


fn compute_screen_coordinates(
    film_aperture_width: f32, film_aperture_height: f32,
    image_width: usize, image_height: usize, near_clipping_plane: f32, focal_len: f32) -> (f32, f32, f32, f32) {

    let film_aspect_ratio = film_aperture_width / film_aperture_height;
    let device_aspect_ratio = image_width as f32 / image_height as f32;

    let mut top = ((film_aperture_height * inch_to_mm / 2.0) / focal_len) * near_clipping_plane;
    let mut right = ((film_aperture_width * inch_to_mm / 2.0) / focal_len) * near_clipping_plane;

    // field of view (horizontal)
    let fov = 2.0 * 180.0 / PI * ((film_aperture_width * inch_to_mm / 2.0) / focal_len).atan();

    let mut xscale = 1.0;
    let mut yscale = 1.0;

    if (film_aspect_ratio > device_aspect_ratio) {
        yscale = film_aspect_ratio / device_aspect_ratio;
    } else {
        xscale = device_aspect_ratio / film_aspect_ratio;
    }

    right *= xscale;
    top *= yscale;

    let bottom = -top;
    let left = -right;

    (top, bottom, left, right)
}


fn convert_to_raster(
    vertex_obj: &Point, object_to_camera: &AffineMat3, l: f32, r: f32, t: f32, b: f32,
    near: f32, image_width: usize, image_height: usize) -> Point {

    // To camera space
    let mut result = object_to_camera * vertex_obj;

    // To clip space
    // 1. Apply perspective
    result.x = near * result.x / -result.z;
    result.y = near * result.y / -result.z;
    // 2.  Convert to [-1, 1]
    result.x = 2.0 * result.x / (r - l) - (r + l) / (r - l);
    result.y = 2.0 * result.y / (t - b) - (t + b) / (t - b);

    // To screen space, i.e [0, w] and [0, h]
    result.x = (result.x + 1.0) * 0.5 * (image_width as f32);
    result.y = (1.0 - result.y) * 0.5 * (image_height as f32);
    result.z = -result.z;

    result
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
                },
                _ => {}
            }
        },
        _ => {},
    }

    if (!state.arcball_enabled) {
        return;
    }

    if (state.curr_mouse_x == app.mouse.x && state.curr_mouse_y == app.mouse.y) {
        return;
    }

    let prev_arcball_vec = Camera::compute_arcball_vector_for_xy(state.curr_mouse_x, state.curr_mouse_y);
    let curr_arcball_vec = Camera::compute_arcball_vector_for_xy(app.mouse.x, app.mouse.y);
    let angle = 2.0 * prev_arcball_vec.dot_product(&curr_arcball_vec).min(1.0).acos();
    let world_to_camera = state.camera.compute_view_matrix();
    let object_to_camera = &world_to_camera * &state.object_to_world;
    let camera_to_object = &world_to_camera.compute_inverse();
    let axis_camera = &prev_arcball_vec.cross_product(&curr_arcball_vec);
    let axis_object = (camera_to_object * axis_camera).normalize();
    let rotation = AffineMat3::rotation(angle, &axis_object);

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
    let (models, _) = tobj::load_obj(&obj_file, true).expect("Failed to load file");

    app
        .new_window()
        .title("CS248 Computer Graphics")
        .size(WIDTH as u32, HEIGHT as u32)
        .view(render_and_display)
        .build()
        .unwrap();

    let mut state = init_state(models[0].clone());

    println!("Mouse pos before: {} {} {} {}", state.curr_mouse_x, state.curr_mouse_y, app.mouse.x, app.mouse.y);

    (*app.main_window()).set_cursor_position_points(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    state.curr_mouse_x = app.mouse.x;
    state.curr_mouse_y = app.mouse.y;

    println!("Mouse pos after: {} {} {} {}", state.curr_mouse_x, state.curr_mouse_y, app.mouse.x, app.mouse.y);

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
    let near_clipping_plane = 1.0;
    let far_clipping_plane = 1000.0;
    let focal_len = 20.0; // in mm
    let film_aperture_width = 0.980; // 35mm Full Aperture in inches
    let film_aperture_width = 0.735;

    // let world_to_camera = Matrix4::new(
    //     0.707107, 0.0, -0.707107, -1.63871,
    //     -0.331295, 0.883452, -0.331295, -5.747777,
    //     0.624695, 0.468521, 0.624695, -40.400412,
    //     0.0, 0.0, 0.0, 1.0
    // );
    // let world_to_camera = Matrix4::new(
    //     1.0, 0.0, 0.0, 0.0,
    //     0.0, 1.0, 0.0, 0.0,
    //     0.0, 0.0, 1.0, -40.400412,
    //     0.0, 0.0, 0.0, 1.0
    // );
    let world_to_camera = state.camera.compute_view_matrix();
    let object_to_camera = &world_to_camera * &state.object_to_world;
    let light_pos_camera = &world_to_camera * &state.light_position;
    let camera_to_object = object_to_camera.compute_inverse();
    let image_width: usize = WIDTH;
    let image_height: usize = HEIGHT;

    let (t, b, l, r) = compute_screen_coordinates(
        film_aperture_width, film_aperture_width, image_width,
        image_height, near_clipping_plane, focal_len);

    let mut frame_buffer = vec![Color::new(0.2, 0.2, 0.2); image_width * image_height];
    let mut z_buffer = vec![far_clipping_plane; image_width * image_height];

    let start = Instant::now();
    let mut preparation_time = Duration::from_secs(0);
    let mut inner_loop_time = Duration::from_secs(0);

    for i in 0..(num_triangles as usize) {
        let preparation_start = Instant::now();

        let idx_1 = model.mesh.indices[i * 3 + 0] as usize;
        let idx_2 = model.mesh.indices[i * 3 + 1] as usize;
        let idx_3 = model.mesh.indices[i * 3 + 2] as usize;

        let v0 = Point::new(model.mesh.positions[idx_1 * 3 + 0], model.mesh.positions[idx_1 * 3 + 1], model.mesh.positions[idx_1 * 3 + 2]);
        let v1 = Point::new(model.mesh.positions[idx_2 * 3 + 0], model.mesh.positions[idx_2 * 3 + 1], model.mesh.positions[idx_2 * 3 + 2]);
        let v2 = Point::new(model.mesh.positions[idx_3 * 3 + 0], model.mesh.positions[idx_3 * 3 + 1], model.mesh.positions[idx_3 * 3 + 2]);

        // Convert the vertices of the triangle to raster space
        let mut v0_raster = convert_to_raster(&v0, &object_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);
        let mut v1_raster = convert_to_raster(&v1, &object_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);
        let mut v2_raster = convert_to_raster(&v2, &object_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);

        // Precompute reciprocal of vertex z-coordinate
        v0_raster.z = 1.0 / v0_raster.z;
        v1_raster.z = 1.0 / v1_raster.z;
        v2_raster.z = 1.0 / v2_raster.z;

        // Gouraud shading colors
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
        let colors_gouraud = (
            face_normal_camera.dot_product(&light_dirs.0),
            face_normal_camera.dot_product(&light_dirs.1),
            face_normal_camera.dot_product(&light_dirs.2),
        );

        // Making face culling
        let v0_view_direction = (-&Vec3::new(v0_camera.x, v0_camera.y, v0_camera.z)).normalize();
        if v0_view_direction.dot_product(&face_normal_camera) < 0.0 {
            continue;
        }

        // Prepare vertex attributes. Divde them by their vertex z-coordinate
        // (though we use a multiplication here because v.z = 1 / v.z)
        let mut st0;
        let mut st1;
        let mut st2;

        if tex.is_empty() {
            st0 = (0.0, 0.0);
            st1 = (0.0, 0.0);
            st2 = (0.0, 0.0);
        } else {
            st0 = (tex[idx_1 * 2], tex[idx_1 * 2 + 1]);
            st1 = (tex[idx_2 * 2], tex[idx_2 * 2 + 1]);
            st2 = (tex[idx_3 * 2], tex[idx_3 * 2 + 1]);

            st0.0 *= v0_raster.z;
            st0.1 *= v0_raster.z;
            st1.0 *= v1_raster.z;
            st1.1 *= v1_raster.z;
            st2.0 *= v2_raster.z;
            st2.1 *= v2_raster.z;
        }

        let x_min = min_of_three(v0_raster.x, v1_raster.x, v2_raster.x);
        let y_min = min_of_three(v0_raster.y, v1_raster.y, v2_raster.y);
        let x_max = max_of_three(v0_raster.x, v1_raster.x, v2_raster.x);
        let y_max = max_of_three(v0_raster.y, v1_raster.y, v2_raster.y);

        // the triangle is out of screen
        if (x_min > (image_width - 1) as f32 || x_max < 0.0 || y_min > (image_height - 1) as f32 || y_max < 0.0) {
            continue;
        }

        // be careful x_min/x_max/y_min/y_max can be negative. Don't cast to uint32_t
        let x0 = cmp::max(0, (x_min.floor() as i32)) as usize;
        let x1 = cmp::min(image_width as i32 - 1, (x_max.floor() as i32)) as usize;
        let y0 = cmp::max(0, (y_min.floor() as i32)) as usize;
        let y1 = cmp::min(image_height as i32 - 1, (y_max.floor() as i32)) as usize;

        let area = edge_function(&v0_raster, &v1_raster, &v2_raster);

        preparation_time += preparation_start.elapsed();

        for y in y0..(y1 + 1) {
            for x in x0..(x1 + 1) {
                let pixel_pos = Point::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let bar_coords = (
                    edge_function(&v1_raster, &v2_raster, &pixel_pos) / area,
                    edge_function(&v2_raster, &v0_raster, &pixel_pos) / area,
                    edge_function(&v0_raster, &v1_raster, &pixel_pos) / area,
                );

                if (bar_coords.0 >= 0.0 && bar_coords.1 >= 0.0 && bar_coords.2 >= 0.0) {
                    let depth = 1.0 / (v0_raster.z * bar_coords.0 + v1_raster.z * bar_coords.1 + v2_raster.z * bar_coords.2);

                    // Depth-buffer test
                    if (depth < z_buffer[y * image_width + x]) {
                        z_buffer[y * image_width + x] = depth;

                        // let tex_coords = (
                        //     (st0.0 * bar_coords.0 + st1.0 * bar_coords.1 + st2.0 * bar_coords.2) * depth,
                        //     (st0.1 * bar_coords.0 + st1.1 * bar_coords.1 + st2.1 * bar_coords.2) * depth
                        // );

                        // Compute the face normal which is used for a simple facing ratio.
                        // Keep in mind that we are doing all calculation in camera space.
                        // Thus the view direction can be computed as the point on the object
                        // in camera space minus Vec3f(0), the position of the camera in camera space.
                        // let view_direction = (-&Vec3::new(pos_camera.x, pos_camera.y, pos_camera.z)).normalize();
                        // let mut n_dot_view = face_normal_camera.dot_product(&view_direction).max(0.0);

                        let mut color;
                        if state.is_gouraud_shading {
                            color = colors_gouraud.0 * bar_coords.0 + colors_gouraud.1 * bar_coords.1 + colors_gouraud.2 * bar_coords.2;
                        } else {
                            let px = (v0_camera.x / -v0_camera.z) * bar_coords.0 + (v1_camera.x / -v1_camera.z) * bar_coords.1 + (v2_camera.x / -v2_camera.z) * bar_coords.2;
                            let py = (v0_camera.y / -v0_camera.z) * bar_coords.0 + (v1_camera.y / -v1_camera.z) * bar_coords.1 + (v2_camera.y / -v2_camera.z) * bar_coords.2;
                            let pos_camera = Point::new(px * depth, py * depth, -depth); // fragmet position is in the camera space
                            let light_dir = (&light_pos_camera - &pos_camera).normalize();

                            color = face_normal_camera.dot_product(&light_dir);
                        }

                        // // The final color is the reuslt of the faction ration multiplied by the
                        // // checkerboard pattern.
                        // let M = 10.0;
                        // let checker = (((tex_coords.0 * M) % 1.0) > 0.5) ^ (((tex_coords.1 * M) % 1.0) < 0.5);
                        // let c = if checker { 0.7 } else { 0.3 };
                        // n_dot_view *= c;

                        frame_buffer[y * image_width + x] = Color::new(color, color, color);
                    }
                }
            }
        }
    }

    let duration = start.elapsed();
    println!("Rasterizer done! Took time: {} ms", duration.as_millis());
    // println!("Preparation time took: {} ms", preparation_time.as_millis());

    let mut img = RgbImage::new(image_width as u32, image_height as u32);
    for y in 0..image_height {
        for x in 0..image_width {
            img.put_pixel(x as u32, y as u32, frame_buffer[image_width * y + x].clone().into());
        }
    }

    let img = DynamicImage::ImageRgb8(img);
    // img.save("image.tga").unwrap();

    img
}


fn init_state(model: Model) -> State {
    println!("Building model!");
    let distance = -300.0;

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
            eye: Point::new(0.0, 0.0, -distance),
            look_at: Point::new(0.0, 0.0, -distance - 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        },
        curr_mouse_x: 0.0,
        curr_mouse_y: 0.0,
        arcball_enabled: false,
        light_position: Point::new(0.0, 100.0, 0.0),
        is_gouraud_shading: true,
    }
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