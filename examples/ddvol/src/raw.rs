//! Importers for RAW volume files. A raw volume file is the raw bytes of
//! volume data in little endian, with X increasing first, then Y, then Z.

use std::io;
use std::io::prelude::*;
use std::fs::File;

use vol::Volume;

//pub fn import(path: &Path, dims: (usize, usize, usize), data_type: &str) -> Volume {
//}

