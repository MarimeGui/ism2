use std::io::{Read, Seek};
use Result;

pub struct JointDefinition {}

impl JointDefinition {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &Vec<String>,
    ) -> Result<JointDefinition> {
        unimplemented!();
    }
}
