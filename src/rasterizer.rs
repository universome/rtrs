use std::time::{Instant, Duration};
use std::cmp;
use std::f32::consts::{PI};
use std::env;

use nalgebra::{Matrix4, Point3, Vector3};
use nannou::prelude::*;
use nannou::image::{DynamicImage, RgbImage};
use tobj::{Model};

// use crate::mesh::{Triangle};
use crate::matrix::*;
use crate::basics::*;

const inch_to_mm: f32 = 25.4;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 640;

pub struct State {
    model: Model,
    // is_mouse_inited: bool,
    // curr_mouse_x: f32,
    // curr_mouse_y: f32,
    // mouse_sensitivity: f32,
    // move_speed: f32,
    // mouse_is_in_window: bool,
    // scroll_speed: f32,
    // rotation_speed: f32,
    // scale_speed: f32,
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


pub fn convert_to_raster(
    vertex: &Point3<f32>, world_to_camera: &Matrix4<f32>, l: f32, r: f32, t: f32, b: f32,
    near: f32, image_width: usize, image_height: usize) -> Point3<f32> {

    let mut result = world_to_camera.transform_point(vertex);

    // convert to screen space
    result[0] = near * result[0] / -result[2];
    result[1] = near * result[1] / -result[2];

    // now convert point from screen space to NDC space (in range [-1,1])
    result[0] = 2.0 * result[0] / (r - l) - (r + l) / (r - l);
    result[1] = 2.0 * result[1] / (t - b) - (t + b) / (t - b);

    // convert to raster space
    result[0] = (result[0] + 1.0) / 2.0 * (image_width as f32);
    result[1] = (1.0 - result[1]) / 2.0 * (image_height as f32); // in raster space y is down so invert direction
    result[2] = -result[2];

    return result
}


pub fn launch(model: Model) {
    nannou::app(init_app).event(update_on_event).run();
}


fn update_on_event(app: &App, state: &mut State, event: Event) {
    // pass
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
        .size(WIDTH, HEIGHT)
        .view(render_and_display)
        .build()
        .unwrap();

    init_state(models[0].clone())
}


fn render_and_display(app: &App, state: &State, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();
    let img = render_state(state);
    let texture = wgpu::Texture::from_image(app, &img);

    draw.texture(&texture);
    draw.to_frame(app, &frame).unwrap();
}


pub fn render_state(state: &State) -> DynamicImage{
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
    let world_to_camera = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, -40.400412,
        0.0, 0.0, 0.0, 1.0
    );
    // let image_width: usize = 1280;
    // let image_height: usize = 960;
    let image_width: usize = 640;
    let image_height: usize = 480;

    let (t, b, l, r) = compute_screen_coordinates(
        film_aperture_width, film_aperture_width, image_width,
        image_height, near_clipping_plane, focal_len);

    // Vec3<unsigned char> *frameBuffer = new Vec3<unsigned char>[image_width * image_height];
    // let mut frame_buffer = [Color::new(0.0, 0.0, 0.0); (image_width * image_height) as usize];
    // let mut z_buffer = [far_clipping_plane; (image_width * image_height) as usize];
    let mut frame_buffer = vec![Color::new(1.0, 1.0, 1.0); image_width * image_height];
    let mut z_buffer = vec![far_clipping_plane; image_width * image_height];

    let mut counter = 0;
    let start = Instant::now();
    let mut preparation_time = Duration::from_secs(0);
    let mut inner_loop_time = Duration::from_secs(0);

    for i in 0..(num_triangles as usize) {
        let preparation_start = Instant::now();
        let idx_1 = model.mesh.indices[i * 3] as usize;
        let idx_2 = model.mesh.indices[i * 3 + 1] as usize;
        let idx_3 = model.mesh.indices[i * 3 + 2] as usize;

        let v0 = (model.mesh.positions[idx_1 * 3 + 0], model.mesh.positions[idx_1 * 3 + 1], model.mesh.positions[idx_1 * 3 + 2]);
        let v1 = (model.mesh.positions[idx_2 * 3 + 0], model.mesh.positions[idx_2 * 3 + 1], model.mesh.positions[idx_2 * 3 + 2]);
        let v2 = (model.mesh.positions[idx_3 * 3 + 0], model.mesh.positions[idx_3 * 3 + 1], model.mesh.positions[idx_3 * 3 + 2]);

        let v0 = Point3::new(v0.0, v0.1, v0.2);
        let v1 = Point3::new(v1.0, v1.1, v1.2);
        let v2 = Point3::new(v2.0, v2.1, v2.2);

        // Convert the vertices of the triangle to raster space
        let mut v0_raster = convert_to_raster(&v0, &world_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);
        let mut v1_raster = convert_to_raster(&v1, &world_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);
        let mut v2_raster = convert_to_raster(&v2, &world_to_camera, l, r, t, b, near_clipping_plane, image_width, image_height);

        // Precompute reciprocal of vertex z-coordinate
        v0_raster[2] = 1.0 / v0_raster[2];
        v1_raster[2] = 1.0 / v1_raster[2];
        v2_raster[2] = 1.0 / v2_raster[2];

        // Prepare vertex attributes. Divde them by their vertex z-coordinate
        // (though we use a multiplication here because v[2] = 1 / v[2])
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

            st0.0 *= v0_raster[2];
            st0.1 *= v0_raster[2];
            st1.0 *= v1_raster[2];
            st1.1 *= v1_raster[2];
            st2.0 *= v2_raster[2];
            st2.1 *= v2_raster[2];
        }

        let x_min = min_of_three(v0_raster[0], v1_raster[0], v2_raster[0]);
        let y_min = min_of_three(v0_raster[1], v1_raster[1], v2_raster[1]);
        let x_max = max_of_three(v0_raster[0], v1_raster[0], v2_raster[0]);
        let y_max = max_of_three(v0_raster[1], v1_raster[1], v2_raster[1]);

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
                let inner_loop_stuff_start = Instant::now();
                let pixel_pos = Point3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let mut w0 = edge_function(&v1_raster, &v2_raster, &pixel_pos);
                let mut w1 = edge_function(&v2_raster, &v0_raster, &pixel_pos);
                let mut w2 = edge_function(&v0_raster, &v1_raster, &pixel_pos);
                inner_loop_time += inner_loop_stuff_start.elapsed();

                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) {
                    w0 /= area;
                    w1 /= area;
                    w2 /= area;
                    let one_div_z = v0_raster[2] * w0 + v1_raster[2] * w1 + v2_raster[2] * w2;
                    let z = 1.0 / one_div_z;

                    // Depth-buffer test
                    if (z < z_buffer[y * image_width + x]) {
                        z_buffer[y * image_width + x] = z;

                        let tex_coords = (
                            (st0.0 * w0 + st1.0 * w1 + st2.0 * w2) * z,
                            (st0.1 * w0 + st1.1 * w1 + st2.1 * w2) * z
                        );

                        // println!("Tex: {}, {}", tex_coords.0, tex_coords.1);

                        // If you need to compute the actual position of the shaded
                        // point in camera space. Proceed like with the other vertex attribute.
                        // Divide the point coordinates by the vertex z-coordinate then
                        // interpolate using barycentric coordinates and finally multiply by sample depth.
                        let v0_cam = world_to_camera.transform_point(&v0);
                        let v1_cam = world_to_camera.transform_point(&v1);
                        let v2_cam = world_to_camera.transform_point(&v2);

                        let px = (v0_cam[0]/-v0_cam[2]) * w0 + (v1_cam[0]/-v1_cam[2]) * w1 + (v2_cam[0]/-v2_cam[2]) * w2;
                        let py = (v0_cam[1]/-v0_cam[2]) * w0 + (v1_cam[1]/-v1_cam[2]) * w1 + (v2_cam[1]/-v2_cam[2]) * w2;
                        let pt = Point3::new(px * z, py * z, -z); // pt is in camera space

                        // Compute the face normal which is used for a simple facing ratio.
                        // Keep in mind that we are doing all calculation in camera space.
                        // Thus the view direction can be computed as the point on the object
                        // in camera space minus Vec3f(0), the position of the camera in camera space.
                        let normal = (&((v1_cam - v0_cam).cross(&(v2_cam - v0_cam)))).normalize();
                        let view_direction = (&-Vector3::new(pt[0], pt[1], pt[2])).normalize();
                        let mut n_dot_view = normal.dot(&view_direction).max(0.0);

                        // The final color is the reuslt of the faction ration multiplied by the
                        // checkerboard pattern.
                        let M = 10.0;
                        let checker = (((tex_coords.0 * M) % 1.0) > 0.5) ^ (((tex_coords.1 * M) % 1.0) < 0.5);
                        let c = if checker { 0.7 } else { 0.3 };
                        n_dot_view *= c;
                        frame_buffer[y * image_width + x].r = n_dot_view;
                        frame_buffer[y * image_width + x].g = n_dot_view;
                        frame_buffer[y * image_width + x].b = n_dot_view;
                    }
                }
            }
        }
    }

    println!("Counter: {}", counter);

    let duration = start.elapsed();
    println!("Rasterizer done! Took time: {} ms", duration.as_millis());
    println!("Preparation time took: {} ms", preparation_time.as_millis());
    println!("Inner loop time took: {} ms", inner_loop_time.as_millis());

    let mut img = RgbImage::new(image_width as u32, image_height as u32);
    for y in 0..image_height {
        for x in 0..image_width {
            img.put_pixel(x as u32, y as u32, frame_buffer[image_width * y + x].clone().into());
        }
    }

    let img = DynamicImage::ImageRgb8(img);
    img.save("image.tga").unwrap();

    img
}


fn init_state(model: Model) -> State {
    println!("Building model!");

    State {
        model: model,
    }
}


#[inline]
fn edge_function(u: &Point3<f32>, v: &Point3<f32>, point: &Point3<f32>) -> f32 {
    // Given two vectors u, v, computes the edge function for the given point
    (point[0] - u[0]) * (v[1] - u[1]) - (point[1] - u[1]) * (v[0] - u[0])
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