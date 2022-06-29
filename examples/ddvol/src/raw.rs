//! Importers for RAW volume files. A raw volume file is the raw bytes of
//! volume data in little endian, with X increasing first, then Y, then Z.

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter;
use std::mem;
use std::path::Path;

use num::{self, NumCast};

use vec3::Vec3i;
use vol::Volume;

/// Import a RAW volume file with the set dimensions. The volume data is
/// expected to be of type T which should be a primitive scalar type
pub fn import<T: NumCast>(path: &Path, dims: Vec3i) -> Volume {
    let mut f = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(e) => panic!("Error opening volume `{:?}`: {}", path, e),
    };
    let mut data: Vec<_> = iter::repeat(0u8)
        .take((dims.x * dims.y * dims.z) as usize * mem::size_of::<T>())
        .collect();
    f.read_exact(&mut data[..])
        .expect("Failed to read entire RAW volume");
    let data: Vec<f32> = data
        .chunks(mem::size_of::<T>())
        .map(|x| unsafe { mem::transmute_copy::<u8, T>(&x[0]) })
        .map(|x| num::cast(x).unwrap())
        .collect();
    let mut volume = Volume::new(dims);
    volume.set_region(&data[..], Vec3i::broadcast(0), dims);
    volume
}
