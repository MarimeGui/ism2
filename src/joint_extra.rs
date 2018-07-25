use std::io::{Read, Seek};
use Result;

pub struct JointExtra {}

impl JointExtra {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &Vec<String>,
    ) -> Result<JointExtra> {
        unimplemented!();
    }
}
