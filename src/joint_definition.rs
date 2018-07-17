use CResult;
use std::io::{Read, Seek};

pub struct JointDefinition {

}

impl JointDefinition {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &Vec<String>) -> CResult<JointDefinition> {
        unimplemented!();
    }
}