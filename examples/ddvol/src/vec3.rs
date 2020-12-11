#![allow(dead_code)]

use std::f32;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Set a nicer type alias for the exported ISPC struct
pub type Vec3f = ::ddvol::Vec3f;
pub type Vec3i = ::ddvol::Vec3i;

impl Vec3f {
    pub fn broadcast(x: f32) -> Vec3f {
        Vec3f { x: x, y: x, z: x }
    }
    pub fn new(x: f32, y: f32, z: f32) -> Vec3f {
        Vec3f { x: x, y: y, z: z }
    }
    pub fn dot(&self, b: &Vec3f) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub fn length(&self) -> f32 {
        f32::sqrt(self.dot(&self))
    }
    pub fn normalized(&self) -> Vec3f {
        let inv_len = 1.0 / self.length();
        Vec3f::new(self.x * inv_len, self.y * inv_len, self.z * inv_len)
    }
    pub fn cross(&self, b: &Vec3f) -> Vec3f {
        Vec3f::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
}

impl Neg for Vec3f {
    type Output = Vec3f;

    fn neg(self) -> Vec3f {
        Vec3f {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for Vec3f {
    type Output = Vec3f;
    /// Add two vectors together
    fn add(self, rhs: Vec3f) -> Vec3f {
        Vec3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3f {
    type Output = Vec3f;
    /// Subtract two vectors
    fn sub(self, rhs: Vec3f) -> Vec3f {
        Vec3f {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vec3f {
    type Output = Vec3f;
    /// Multiply two vectors
    fn mul(self, rhs: Vec3f) -> Vec3f {
        Vec3f {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f32> for Vec3f {
    type Output = Vec3f;
    /// Scale the vector by some value
    fn mul(self, rhs: f32) -> Vec3f {
        Vec3f {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3f> for f32 {
    type Output = Vec3f;
    /// Scale the vector by some value
    fn mul(self, rhs: Vec3f) -> Vec3f {
        Vec3f {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div for Vec3f {
    type Output = Vec3f;
    /// Divide the vectors components by the right hand side's components
    fn div(self, rhs: Vec3f) -> Vec3f {
        Vec3f {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f32> for Vec3f {
    type Output = Vec3f;
    /// Divide the vectors components by a scalar
    fn div(self, rhs: f32) -> Vec3f {
        Vec3f {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Vec3i {
    pub fn broadcast(x: i32) -> Vec3i {
        Vec3i { x: x, y: x, z: x }
    }
    pub fn new(x: i32, y: i32, z: i32) -> Vec3i {
        Vec3i { x: x, y: y, z: z }
    }
}

impl Neg for Vec3i {
    type Output = Vec3i;

    fn neg(self) -> Vec3i {
        Vec3i {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for Vec3i {
    type Output = Vec3i;
    /// Add two vectors together
    fn add(self, rhs: Vec3i) -> Vec3i {
        Vec3i {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3i {
    type Output = Vec3i;
    /// Subtract two vectors
    fn sub(self, rhs: Vec3i) -> Vec3i {
        Vec3i {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vec3i {
    type Output = Vec3i;
    /// Multiply two vectors
    fn mul(self, rhs: Vec3i) -> Vec3i {
        Vec3i {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<i32> for Vec3i {
    type Output = Vec3i;
    /// Scale the vector by some value
    fn mul(self, rhs: i32) -> Vec3i {
        Vec3i {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3i> for i32 {
    type Output = Vec3i;
    /// Scale the vector by some value
    fn mul(self, rhs: Vec3i) -> Vec3i {
        Vec3i {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}
