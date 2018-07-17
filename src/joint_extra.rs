use CResult;
use std::io::{Read, Seek};

pub struct JointExtra {}

impl JointExtra {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &Vec<String>) -> CResult<JointExtra> {
        unimplemented!();
    }
}