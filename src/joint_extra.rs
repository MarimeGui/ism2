use crate::error::ISM2ImportError;
use ez_io::{MagicNumberCheck, ReadE};
use std::io::{Read, Seek, SeekFrom};
use crate::Result;

pub struct JointExtra {
    pub sub_sections: Vec<Unnamed31>,
}

pub struct Unnamed31 {
    pub name1: String,
    pub name2: String,
    pub sub_sections: Vec<Unnamed30>,
}

pub struct Unnamed30 {
    pub identity_matrix: [f32; 16],
    pub sub_sections: Vec<Buffer>,
}

pub struct Buffer {
    pub data: BufferData,
}

pub enum BufferData {
    BoneNames(Vec<String>),
    InverseBindMatrices(Vec<[f32; 16]>),
}

impl JointExtra {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &[String]) -> Result<JointExtra> {
        reader.check_magic_number(&[0x32, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00])?; // Magic Number + 0x14
        let nb_sub_sections = reader.read_le_to_u32()?;
        reader.check_magic_number(&[0u8; 8])?;
        let mut sub_section_offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            sub_section_offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in sub_section_offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(Unnamed31::import(reader, strings_table)?);
        }
        Ok(JointExtra { sub_sections })
    }
}

impl Unnamed31 {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &[String]) -> Result<Unnamed31> {
        reader.check_magic_number(&[0x31, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00])?; // Magic Number + 0x14
        let nb_sub_sections = reader.read_le_to_u32()?;
        let name1_id = reader.read_le_to_u32()?;
        let name2_id = reader.read_le_to_u32()?;
        let mut sub_sections_offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            sub_sections_offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in sub_sections_offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(Unnamed30::import(reader, strings_table)?);
        }
        Ok(Unnamed31 {
            name1: strings_table[name1_id as usize].clone(),
            name2: strings_table[name2_id as usize].clone(),
            sub_sections,
        })
    }
}

impl Unnamed30 {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &[String]) -> Result<Unnamed30> {
        reader.check_magic_number(&[0x30, 0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00])?; // Magic Number + 0x54
        let nb_sub_sections = reader.read_le_to_u32()?;
        let _unk1 = reader.read_le_to_u32()?;
        reader.check_magic_number(&[0u8; 4])?;
        let mut identity_matrix = [0f32; 16];
        for i in 0..16 {
            identity_matrix[i] = reader.read_le_to_f32()?;
        }
        let mut sub_sections_offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            sub_sections_offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in sub_sections_offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(Buffer::import(reader, strings_table)?);
        }
        Ok(Unnamed30 {
            identity_matrix,
            sub_sections,
        })
    }
}

impl Buffer {
    pub fn import<R: Read + Seek>(reader: &mut R, strings_table: &[String]) -> Result<Buffer> {
        reader.check_magic_number(&[0x44, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00])?; // Magic Number + 0x20
        let nb_entries = reader.read_le_to_u32()?;
        reader.check_magic_number(&[0u8; 4])?;
        let part1 = reader.read_le_to_u32()?;
        let part2 = reader.read_le_to_u32()?;
        let part3 = reader.read_le_to_u32()?;
        reader.check_magic_number(&[0u8; 4])?;
        let data;
        match (part1, part2, part3) {
            (0x05, 0x01, 0x00) => {
                let mut strings = Vec::with_capacity(nb_entries as usize);
                for _ in 0..nb_entries {
                    let id = reader.read_le_to_u16()?;
                    let name = strings_table[id as usize].clone();
                    strings.push(name);
                }
                data = BufferData::BoneNames(strings);
            }
            (0x0C, 0x10, 0x10) => {
                let mut matrices = Vec::with_capacity((nb_entries / 16) as usize);
                for _ in 0..(nb_entries / 16) {
                    let mut matrix = [0f32; 16];
                    for i in 0..16 {
                        matrix[i] = reader.read_le_to_f32()?;
                    }
                    matrices.push(matrix);
                }
                data = BufferData::InverseBindMatrices(matrices);
            }
            _ => return Err(ISM2ImportError::UnrecognizedBufferType),
        }
        Ok(Buffer { data })
    }
}
