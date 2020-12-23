# RtRs
Simple ray tracing & rasterization engine written in rust.

![Newell Teaset (ray tracing example)](./newell-teaset.png "Newell Teaset (ray tracing example)")

# Features
Ray Tracing:
- [x] Rendering quadrics: sphers, cones, cones (with slabs),
- [x] Rendering meshes (passed as .obj files). Also precomputing vertex normals
- [x] Bounding Volumes Hierarchy
- [x] Full camera movement + zoom
- [x] Precomputed mesh normals
- [x] Object rotations
- [x] Parallel execution
- [x] Lambertian/Phong shading
- [x] Gouraud/Phong shading
- [x] Antialiasing via supersampling (via distributed ray tracing)
- [x] Soft shadows (via distributed ray tracing)
- [x] Reflections + glossy reflections (via distributed ray tracing)
- [ ] Refraction & attenutation

Rasterization:
- [x] Mesh rasterization
- [x] Object rotation
- [x] Lambertian/Phong shading
- [x] Gouraud/Phong shading
- [x] Texture mapping with stripe effect
- [x] Backface culling
- [x] Full camera movement + zoom
- [x] Simple antialising (via supersampling)
- [ ] Clipping
- [ ] Camera movement

### How to run the code
You need to [install Rust on your system](https://www.rust-lang.org/tools/install) and then just type the command:
```
cargo run --release
```

The binary file will be located at `target/release/rtrs`.
There is also a binary attached which is located at `rtrs` (Note: compiled on OS X Catalina 10.15.6).

### Core dependencies
- [tobj](https://github.com/Twinklebear/tobj) to load .obj files
- [nannou](http://nannou.cc/) to open a window and detect key press events, which is an analog of OpenFrameworks for Rust language
- [image](https://crates.io/crates/image) to save an image
- [rayon](https://crates.io/crates/rayon) which provides easy-to-use parallelism for Rust

### Disclaimer
All experiments are done on MacBook Pro 16' with 2,3 GHz 8-Core Intel Core i9.
