use half::f16;
use std::io::{Read, Seek};
use Result;

pub struct JointDefinition {}

pub enum JointDefinitionSubSection {
    Unnamed04,
    Joint(Joint),
}

// pub struct Unnamed04 {}

// pub enum Unnamed04SubSection {
//     Unnamed5B(Unnamed5B),
//     Unnamed4C(Unnamed4C),
// }

// pub struct Unnamed5B {}

// pub enum Unnamed5BSubSection {
//     Unnamed5F(Unnamed5F),
//     Unnamed5E(Unnamed5E),
//     Unnamed5D(Unnamed5D),
// }

// pub struct Unnamed5F {}

// pub struct Unnamed5E {}

// pub struct Unnamed5D {}

// pub struct Unnamed4C {}

pub struct Joint {}

pub enum JointSubSection {
    Offsets(JointAttributesOffsets),
}

pub struct JointAttributesOffsets {
    // 0x5B
}

pub enum JointAttribute {
    Transform(JointTransform),
    EulerRoll(JointRoll),
    EulerPitch(JointPitch),
    EulerYaw(JointYaw),
}

pub struct JointTransform {
    // 0x14
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct JointRoll {
    // 0x67
    pub angle: f16,
}

pub struct JointPitch {
    // 0x68
    pub angle: f16,
}

pub struct JointYaw {
    // 0x69
    pub angle: f16,
}

impl JointDefinition {
    #[allow(unused_variables)]
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &[String],
    ) -> Result<JointDefinition> {
        unimplemented!();
    }
}
