//! A simple as possible example demonstrating how to use the `draw` API to display a texture.
extern crate nannou;
use nannou::prelude::*;

mod render;


fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    app.new_window().size(512, 512).view(view).build().unwrap();

    let img = render::render();
    let texture = wgpu::Texture::from_image(app, &img);

    Model { texture }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();

    draw.texture(&model.texture);

    println!("{}", app.time);

    draw.to_frame(app, &frame).unwrap();
}
