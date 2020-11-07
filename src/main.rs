extern crate rayon;
extern crate nannou;
extern crate derive_more;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate float_cmp;

use rayon::prelude::*;

mod scene;
mod camera;
mod basics;
mod surface;
mod matrix;
mod ray_tracer;


fn main() {
    // rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    ray_tracer::launch();
}
