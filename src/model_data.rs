use error::{ISM2ImportError, UnknownSubSection};
use ez_io::{MagicNumberCheck, ReadE};
use half::f16;
use std::io::{Read, Seek, SeekFrom};
use Result;

/// Defines all the geometry of the model
pub struct ModelData {
    pub zero_a: Unnamed0A,
}

pub struct Unnamed0A {
    pub sub_sections: Vec<SubSection>,
}

pub enum SubSection {
    Vertices(Vertices),
    Faces(Faces),
    Unnamed6E,
}

pub struct Vertices {
    pub nb_vertices: u32,
    pub sub_sections: Vec<VerticesSubSection>,
}

pub enum VerticesSubSection {
    Unnamed00,
    Unnamed02,
    Unnamed0E,
    Data(Data),
}

pub struct Data {
    pub vertices: Vec<Vertex>,
}

pub struct Vertex {
    pub position_coordinates: [f32; 3],
    pub texture_coordinates: [f16; 2],
}

pub struct Faces {}

pub struct Unnamed45 {}

impl ModelData {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<ModelData> {
        reader.check_magic_number(&[0xB, 0, 0, 0, 0xC, 0, 0, 0, 0x1, 0, 0, 0])?; // Magic Number, 0x0C, Number of sections should be 1
        let section_offset = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Start(u64::from(section_offset)))?;
        let zero_a = Unnamed0A::import(reader)?;
        Ok(ModelData { zero_a })
    }
}

impl Unnamed0A {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<Unnamed0A> {
        reader.check_magic_number(&[0xA, 0, 0, 0, 0x20, 0, 0, 0])?; // Magic number, 0x20
        let nb_sub_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(0x14))?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(SubSection::import(reader)?);
        }
        Ok(Unnamed0A { sub_sections })
    }
}

impl SubSection {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<SubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x59 => SubSection::Vertices(Vertices::import(reader)?),
            0x46 => SubSection::Faces(Faces::import(reader)?),
            0x6E => SubSection::Unnamed6E,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: magic_number,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl Vertices {
    pub fn import<R: Read>(reader: &mut R) -> Result<Vertices> {
        unimplemented!();
    }
}

impl Faces {
    pub fn import<R: Read>(reader: &mut R) -> Result<Faces> {
        unimplemented!();
    }
}
