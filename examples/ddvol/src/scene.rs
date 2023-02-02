//! Provides the scene struct containing information needed to render a
//! volume from some camera position. Scenes are stored in JSON format, for example:
//!
//! ```json
//! {
//!     "volume": {
//!            "file": "csafe-heptane-302-volume.raw",
//!            "data_type": "u8",
//!            "dimensions": [302, 302, 302]
//!        },
//!        "transfer_function": "cool_warm",
//!        "camera": {
//!            "pos": [-0.5, 0.5, 1.5],
//!            "target": [0.5, 0.5, 0.5],
//!            "up": [0, 1, 0],
//!            "fovy": 60,
//!        },
//!        "width": 512,
//!        "height": 512,
//!        "background": [0.1, 0.1, 0.1],
//!        "n_samples": 4
//! }
//! ```
//!
//! The `transfer_function` element can be one of the default ones `jet`, `cool_warm` or
//! `gray_scale` or the name of a ParaView exported transfer function (any string ending in
//! .json). ParaView transfer function importing is still a TODO though.

use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde_json::{self, Value};

use crate::camera::Camera;
use crate::ddvol;
use crate::raw;
use crate::tfn::TransferFunction;
use crate::vec3::{Vec3f, Vec3i};
use crate::vol::Volume;

pub type RenderParams = ddvol::RenderParams;

pub struct Scene {
    /// Image width
    pub width: usize,
    /// Image height
    pub height: usize,
    pub camera: Camera,
    pub volume: Volume,
    pub params: RenderParams,
}

impl Scene {
    pub fn load(file: &str) -> Scene {
        let mut f = match File::open(file) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open scene file: {e}"),
        };
        let mut content = String::new();
        if let Err(e) = f.read_to_string(&mut content) {
            panic!("Failed to read scene file: {e}");
        }
        let data: Value = match serde_json::from_str(&content[..]) {
            Ok(d) => d,
            Err(e) => panic!("JSON parsing error: {e}"),
        };
        if !data.is_object() {
            panic!("Expected a root JSON object. See example scenes");
        }
        let base_path = match Path::new(file).parent() {
            Some(p) => p,
            None => Path::new(file),
        };
        let img_width = data
            .get("width")
            .expect("image width must be set")
            .as_u64()
            .expect("image width must be an int") as usize;
        let img_height = data
            .get("height")
            .expect("image height must be set")
            .as_u64()
            .expect("image height must be an int") as usize;
        let mut volume = Scene::load_volume(
            data.get("volume").expect("A volume must be specified"),
            base_path,
        );
        let tfn = Scene::load_transfer_function(
            data.get("transfer_function")
                .expect("A transfer function must be specified"),
            base_path,
        );
        volume.set_transfer_function(tfn);

        let camera = Scene::load_camera(
            data.get("camera").expect("A camera must be specified"),
            img_width,
            img_height,
        );
        let render_params = Scene::load_render_params(&data);
        Scene {
            width: img_width,
            height: img_height,
            camera,
            volume,
            params: render_params,
        }
    }
    fn load_volume(e: &Value, base_path: &Path) -> Volume {
        let mut vol_file = Path::new(
            e.get("file")
                .expect("A volume filename must be set")
                .as_str()
                .expect("Volume filename must be a string"),
        )
        .to_owned();
        if !vol_file.is_absolute() {
            vol_file = base_path.join(vol_file);
        }
        let dimensions = Scene::load_vec3i(
            e.get("dimensions")
                .expect("Volume dims must be set for RAW volume"),
        )
        .expect("Invalid dimensions specified");

        let dtype = e
            .get("data_type")
            .expect("A data type must be specified for RAW volumes")
            .as_str()
            .expect("data type must be a string");
        if dtype == "u8" {
            raw::import::<u8>(vol_file.as_path(), dimensions)
        } else if dtype == "u16" {
            raw::import::<u16>(vol_file.as_path(), dimensions)
        } else if dtype == "f32" {
            raw::import::<f32>(vol_file.as_path(), dimensions)
        } else if dtype == "f64" {
            raw::import::<f64>(vol_file.as_path(), dimensions)
        } else {
            panic!("Unrecognized data type {dtype}! Valid options are u8, u16, f32, f64");
        }
    }
    fn load_transfer_function(e: &Value, base_path: &Path) -> TransferFunction {
        let tfn_file = Path::new(
            e.as_str()
                .expect("transfer_function filename/name must be a string"),
        );
        // Load the ParaView transfer function file if it's one, otherwise
        // see if it's one of our defaults we can provide
        if tfn_file.extension() == Some(OsStr::new("json")) {
            let mut tfn_buf = tfn_file.to_owned();
            if !tfn_buf.is_absolute() {
                tfn_buf = base_path.join(tfn_buf);
            }
            Scene::import_paraview_tfn(tfn_buf.as_path())
        } else {
            let tfn_name = tfn_file.to_str().unwrap();
            if tfn_name == "grayscale" {
                TransferFunction::grayscale()
            } else if tfn_name == "jet" {
                TransferFunction::jet()
            } else if tfn_name == "cool_warm" {
                TransferFunction::cool_warm()
            } else {
                panic!("Scene error: {tfn_name} is not a built in transfer function");
            }
        }
    }
    fn import_paraview_tfn(path: &Path) -> TransferFunction {
        println!("Importing ParaView transfer function {}", path.display());
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open ParaView transfer function file: {e}"),
        };
        let mut content = String::new();
        if let Err(e) = f.read_to_string(&mut content) {
            panic!("Failed to read ParaView transfer function file: {e}");
        }
        let data: Value = match serde_json::from_str(&content[..]) {
            Ok(d) => d,
            Err(e) => panic!("JSON parsing error: {e}"),
        };
        let pv_tfn = &data
            .as_array()
            .expect("Expected a root JSON array in ParaView function")[0];
        let color_space = pv_tfn
            .get("ColorSpace")
            .expect("Expected a color space from ParaView function")
            .as_str()
            .expect("ColorSpace must be a string");
        if color_space == "Diverging" {
            println!(
                "Warning: ParaView's diverging colormap interpolation is not supported, \
                      you may see some incorrect colors"
            );
        }
        let name = pv_tfn
            .get("Name")
            .expect("Expected a name for ParaView function");
        if pv_tfn.get("Points").is_some() {
            println!(
                "Warning: Opacity values in ParaView transfer functions are currently ignored"
            );
        }
        let rgb_data = pv_tfn
            .get("RGBPoints")
            .expect("Expected RGBPoints specifying the transfer function")
            .as_array()
            .expect("RGBPoints must be an array");
        let rgb_points: Vec<_> = rgb_data
            .chunks(4)
            .map(|x| {
                let val = x[0].as_f64().unwrap() as f32;
                let r = x[1].as_f64().unwrap() as f32;
                let g = x[2].as_f64().unwrap() as f32;
                let b = x[3].as_f64().unwrap() as f32;
                (val, Vec3f::new(r, g, b))
            })
            .collect();
        // Re-sample the ParaView transfer function into an evenly spaced linear transfer function
        let mut colors = Vec::new();
        let mut lo = 0;
        let mut hi = 0;
        colors.push(rgb_points[0].1);
        for i in 1..255 {
            let x = i as f32 / 255.0;
            if x > rgb_points[lo].0 {
                hi = rgb_points.iter().skip(lo).take_while(|pt| x > pt.0).count() + lo;
                lo = hi - 1;
            }
            let delta = x - rgb_points[lo].0;
            let interval = rgb_points[hi].0 - rgb_points[lo].0;
            let col = if delta == 0.0 || interval == 0.0 {
                rgb_points[lo].1
            } else {
                rgb_points[lo].1 + delta / interval * (rgb_points[hi].1 - rgb_points[lo].1)
            };
            colors.push(col);
        }
        colors.push(rgb_points.last().unwrap().1);
        println!("Imported ParaView transfer function {name}");
        TransferFunction::new(&colors[..], &[0.0, 0.5])
    }
    fn load_camera(e: &Value, width: usize, height: usize) -> Camera {
        let pos = Scene::load_vec3f(e.get("pos").expect("Camera view position must be set"))
            .expect("Invalid camera position");
        let target = Scene::load_vec3f(e.get("target").expect("Camera view target must be set"))
            .expect("Invalid camera target");
        let up = Scene::load_vec3f(e.get("up").expect("Camera up vector must be set"))
            .expect("Invalid camera up");
        let fovy = e
            .get("fovy")
            .expect("Camera FOV Y must be set")
            .as_f64()
            .expect("FOV Y must be a float") as f32;
        Camera::new(pos, target, up, fovy, width as u32, height as u32)
    }
    fn load_render_params(e: &Value) -> RenderParams {
        let background =
            Scene::load_vec3f(e.get("background").expect("Background color must be set"))
                .expect("Background color must be a vec3f");
        let n_samples = e
            .get("n_samples")
            .expect("n_samples per pixel must be set")
            .as_i64()
            .expect("n_samples must be an int") as i32;
        RenderParams {
            background,
            n_samples,
        }
    }
    fn load_vec3i(e: &Value) -> Option<Vec3i> {
        e.as_array().map(|x| {
            assert_eq!(x.len(), 3);
            Vec3i::new(
                x[0].as_i64().unwrap() as i32,
                x[1].as_i64().unwrap() as i32,
                x[2].as_i64().unwrap() as i32,
            )
        })
    }
    fn load_vec3f(e: &Value) -> Option<Vec3f> {
        e.as_array().map(|x| {
            assert_eq!(x.len(), 3);
            Vec3f::new(
                x[0].as_f64().unwrap() as f32,
                x[1].as_f64().unwrap() as f32,
                x[2].as_f64().unwrap() as f32,
            )
        })
    }
}
