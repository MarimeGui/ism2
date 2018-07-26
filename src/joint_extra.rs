use std::io::{Read, Seek};
use Result;

pub struct JointExtra {}

impl JointExtra {
    #[allow(unused_variables)]
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &[String]) -> Result<JointExtra> {
        unimplemented!();
    }
}
