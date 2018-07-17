use CResult;
use std::io::{Read, Seek};

pub struct ModelData {}

impl ModelData {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &Vec<String>) -> CResult<ModelData> {
        unimplemented!();
    }
}