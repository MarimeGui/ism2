use error::{ISM2ImportError, UnknownSubSection};
use ez_io::{MagicNumberCheck, ReadE};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use Result;

pub struct JointDefinition {
    pub sub_sections: Vec<JointDefinitionSubSection>,
}

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

pub struct Joint {
    pub name: String,
    pub parent_index: Option<usize>,
    pub sub_sections: Vec<JointSubSection>,
}

pub enum JointSubSection {
    Offsets(JointAttributesOffsets),
    Unnamed5C,
}

pub struct JointAttributesOffsets {
    pub attributes: Vec<JointAttribute>,
}

pub enum JointAttribute {
    Transform(JointTransform),
    EulerRoll(JointRoll),
    EulerPitch(JointPitch),
    EulerYaw(JointYaw),
    Unnamed5D,
    Unnamed5E,
    Unnamed5F,
    Unnamed15,
    Unnamed70,
    Unnamed71,
    Unnamed72,
    Unnamed73,
    Unnamed74,
    Unnamed75,
    Unnamed76,
    Unnamed77,
    Unnamed7A,
    Unnamed7B,
    Unnamed7C,
    Unnamed7D,
    Unnamed7E,
}

pub struct JointTransform {
    // 0x14
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct JointRoll {
    // 0x67
    pub angle: f32,
}

pub struct JointPitch {
    // 0x68
    pub angle: f32,
}

pub struct JointYaw {
    // 0x69
    pub angle: f32,
}

impl JointDefinition {
    // Nodes ??
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &[String],
    ) -> Result<JointDefinition> {
        reader.check_magic_number(&[0x03, 0, 0, 0, 0x14, 0, 0, 0])?;
        let nb_sub_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(0x8))?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        let mut offset_index_map = HashMap::new();
        let mut offset_index_map_counter = 0usize;
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(JointDefinitionSubSection::import(
                reader,
                strings_table,
                &mut offset_index_map,
                &mut offset_index_map_counter,
            )?);
        }
        Ok(JointDefinition { sub_sections })
    }
}

impl JointDefinitionSubSection {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &[String],
        offset_index_map: &mut HashMap<u64, usize>,
        offset_index_map_counter: &mut usize,
    ) -> Result<JointDefinitionSubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x04 => JointDefinitionSubSection::Unnamed04,
            0x05 => {
                offset_index_map.insert(
                    reader.seek(SeekFrom::Current(0))?,
                    *offset_index_map_counter,
                );
                *offset_index_map_counter += 1;
                JointDefinitionSubSection::Joint(Joint::import(
                    reader,
                    strings_table,
                    offset_index_map,
                )?)
            }
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x03,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl Joint {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &[String],
        offset_index_map: &mut HashMap<u64, usize>,
    ) -> Result<Joint> {
        reader.check_magic_number(&[0x05, 0, 0, 0, 0x40, 0, 0, 0])?;
        let nb_sub_sections = reader.read_le_to_u32()?;
        let string_table_index = reader.read_le_to_u32()?;
        let name = strings_table[string_table_index as usize].clone();
        reader.seek(SeekFrom::Current(0xC))?;
        let parent_joint_offset = reader.read_le_to_u32()?;
        let parent_index = if parent_joint_offset == 0 {
            None
        } else {
            match offset_index_map.get(&u64::from(parent_joint_offset)) {
                Some(id) => Some(*id),
                None => panic!("This should not happen"),
            }
        };
        reader.seek(SeekFrom::Current(0x20))?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(JointSubSection::import(reader)?);
        }
        Ok(Joint {
            name,
            parent_index,
            sub_sections,
        })
    }
}

impl JointSubSection {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointSubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x5B => JointSubSection::Offsets(JointAttributesOffsets::import(reader)?),
            0x5C => JointSubSection::Unnamed5C,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x05,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl JointAttributesOffsets {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointAttributesOffsets> {
        reader.check_magic_number(&[0x5B, 0, 0, 0, 0x0C, 0, 0, 0])?;
        let nb_attributes = reader.read_le_to_u32()?;
        let mut offsets = Vec::with_capacity(nb_attributes as usize);
        for _ in 0..nb_attributes {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut attributes = Vec::with_capacity(nb_attributes as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            attributes.push(JointAttribute::import(reader)?);
        }
        Ok(JointAttributesOffsets { attributes })
    }
}

impl JointAttribute {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointAttribute> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x14 => JointAttribute::Transform(JointTransform::import(reader)?),
            0x67 => JointAttribute::EulerRoll(JointRoll::import(reader)?),
            0x68 => JointAttribute::EulerPitch(JointPitch::import(reader)?),
            0x69 => JointAttribute::EulerYaw(JointYaw::import(reader)?),
            0x5D => JointAttribute::Unnamed5D,
            0x5E => JointAttribute::Unnamed5E,
            0x5F => JointAttribute::Unnamed5F,
            0x15 => JointAttribute::Unnamed15,
            0x70 => JointAttribute::Unnamed70,
            0x71 => JointAttribute::Unnamed71,
            0x72 => JointAttribute::Unnamed72,
            0x73 => JointAttribute::Unnamed73,
            0x74 => JointAttribute::Unnamed74,
            0x75 => JointAttribute::Unnamed75,
            0x76 => JointAttribute::Unnamed76,
            0x77 => JointAttribute::Unnamed77,
            0x7A => JointAttribute::Unnamed7A,
            0x7B => JointAttribute::Unnamed7B,
            0x7C => JointAttribute::Unnamed7C,
            0x7D => JointAttribute::Unnamed7D,
            0x7E => JointAttribute::Unnamed7E,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x5B,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl JointTransform {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointTransform> {
        reader.check_magic_number(&[0x14, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(4))?;
        let x = reader.read_le_to_f32()?;
        let y = reader.read_le_to_f32()?;
        let z = reader.read_le_to_f32()?;
        Ok(JointTransform { x, y, z })
    }
}

impl JointRoll {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointRoll> {
        reader.check_magic_number(&[0x67, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(16))?;
        let angle = reader.read_le_to_f32()?;
        Ok(JointRoll { angle })
    }
}

impl JointPitch {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointPitch> {
        reader.check_magic_number(&[0x68, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(16))?;
        let angle = reader.read_le_to_f32()?;
        Ok(JointPitch { angle })
    }
}

impl JointYaw {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<JointYaw> {
        reader.check_magic_number(&[0x69, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(16))?;
        let angle = reader.read_le_to_f32()?;
        Ok(JointYaw { angle })
    }
}
