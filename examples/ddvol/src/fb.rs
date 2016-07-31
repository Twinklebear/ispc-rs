//! Provides a Framebuffer type for writing tiles of the rendered image

use ddvol;

/// An RGBA_F32 framebuffer
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Framebuffer {
        Framebuffer { width: width, height: height, data: vec![0.0; width * height * 4] }
    }
    /// Convert the framebuffer to SRGB8 and return the color buffer
    pub fn srgb8(&self) -> Vec<u8> {
        let mut srgb = vec![0u8; self.width * self.height * 3];
        unsafe {
        ddvol::framebuffer_to_srgb(self.data.as_ptr(), srgb.as_mut_ptr(), self.width as u32, self.height as u32);
        }
        srgb
    }
}

