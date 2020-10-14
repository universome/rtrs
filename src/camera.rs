use crate::basics::*;

#[derive(Debug, Clone, Copy)]
pub enum ProjectionType {Parallel, Perspective}

#[derive(Debug, Clone)]
pub struct Camera {
    // Defaults in world space
    pub origin: Point,
    direction: Vec3,
    up: Vec3,
    right: Vec3,

    projection_type: ProjectionType,
    viewing_plane: ViewingPlane,
}


#[derive(Debug, Clone)]
pub struct ViewingPlane {
    pub z: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub width: u32,
    pub height: u32,
}


impl Camera {
    pub fn from_z_position(z: f32, fov: f32, projection_type: ProjectionType, width: u32, height: u32) -> Camera {
        Camera {
            origin: Point {x: 0.0, y: 0.0, z: z},
            direction: Vec3 {x: 0.0, y: 0.0, z: 1.0},
            up: Vec3 {x: 0.0, y: 1.0, z: 0.0},
            right: Vec3 {x: 1.0, y: 0.0, z: 0.0},
            projection_type: projection_type,
            viewing_plane: ViewingPlane::from_fov(fov, z, width, height),
        }
    }

    pub fn generate_ray(&self, i: u32, j: u32) -> Ray {
        let (u, v) = self.viewing_plane.generate_uv_coords(i, j);
        let d = self.viewing_plane.z - self.origin.z;

        match self.projection_type {
            ProjectionType::Perspective => Ray {
                // TODO: actually, we do not need to clone anything here, right?
                origin: self.origin.clone(),
                direction: &self.direction * (-d) + &self.right * u + &self.up * v
            },
            ProjectionType::Parallel => Ray {
                origin: &(&self.origin + &(&self.right * u)) + &(&self.up * v),
                direction: &self.direction * (-1.0),
            }
        }
    }
}


impl ViewingPlane {
    pub fn from_fov(fov: f32, z: f32, width: u32, height: u32) -> ViewingPlane {
        let y_half = (fov * 0.5).tanh();
        let x_half = y_half * (width as f32) / (height as f32);

        ViewingPlane {
            z: z + 1.0 / (fov * 0.5).tanh(),
            x_min: -x_half,
            x_max: x_half,
            y_min: -y_half,
            y_max: y_half,
            width: width,
            height: height,
        }
    }

    pub fn generate_uv_coords(&self, i: u32, j: u32) -> (f32, f32) {
        let x_dist = self.x_max - self.x_min;
        let y_dist = self.y_max - self.y_min;
        let u = self.x_min + x_dist * (i as f32 + 0.5) / (self.width as f32);
        let v = self.y_min + y_dist * (j as f32 + 0.5) / (self.height as f32);

        (u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewing_plane() {
        let vp = ViewingPlane {
            z: 0.0,
            x_min: -2.0,
            x_max: 2.0,
            y_min: -1.5,
            y_max: 1.5,
            width: 640,
            height: 480,
        };

        assert_eq!(vp.generate_uv_coords(0, 0), (-1.996875, -1.496875));
        assert_eq!(vp.generate_uv_coords(320, 240), (0.0031249523, 0.0031249523));
        assert_eq!(vp.generate_uv_coords(640, 480), (2.0031252, 1.503125));
    }
}
