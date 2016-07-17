//! Importers for RAW volume files. A raw volume file is the raw bytes of
//! volume data in little endian, with X increasing first, then Y, then Z.

use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::mem;
use std::iter;

use num::{self, NumCast};

use vol::Volume;
use vec3::Vec3i;

/// Import a RAW volume file with the set dimensions. The volume data is
/// expected to be of type T which should be a primitive scalar type
pub fn import<T: NumCast>(path: &Path, dims: Vec3i) -> Volume {
    let mut f = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(e) => panic!("Error opening volume {}", e),
    };
    let mut data: Vec<_> = iter::repeat(0u8).take((dims.x * dims.y * dims.z) as usize).collect();
    f.read_exact(&mut data[..]).expect("Failed to read entire RAW volume");
    let data: Vec<f32> = data.chunks(mem::size_of::<T>())
        .map(|x| unsafe { mem::transmute_copy::<u8, T>(&x[0]) })
        .map(|x| num::cast(x).unwrap()).collect();
    let mut volume = Volume::new(dims);
    volume.set_region(&data[..], Vec3i::broadcast(0), dims);
    volume
}

