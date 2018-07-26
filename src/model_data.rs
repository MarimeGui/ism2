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
    Data(VerticesDataBuffer),
    Unnamed07,
    Unnamed01
}

pub struct VerticesDataBuffer {
    pub vertices: Vec<Vertex>,
}

pub struct Vertex {
    pub position_coordinates: [f32; 3],
    pub texture_coordinates: [f16; 2],
}

pub struct Faces {
    pub nb_faces: u32,
    pub sub_sections: Vec<FacesSubSection>,
}

pub enum FacesSubSection {
    Data(FacesDataBuffer),
    Unnamed6E,
}

pub struct FacesDataBuffer {
    pub faces: Vec<Face>,
}

pub struct Face {
    pub points: [u16; 3],
}

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
                    magic_number_section: 0x0A,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl Vertices {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<Vertices> {
        reader.check_magic_number(&[0x59, 0, 0, 0, 0x1C, 0, 0, 0])?; // Magic Number + 0x1C
        let nb_sub_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(4))?;
        let nb_vertices = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(8))?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(VerticesSubSection::import(reader, &nb_vertices)?);
        }
        Ok(Vertices {
            nb_vertices,
            sub_sections,
        })
    }
}

impl VerticesSubSection {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_vertices: &u32) -> Result<VerticesSubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x00 => VerticesSubSection::Unnamed00,
            0x02 => VerticesSubSection::Unnamed02,
            0x0E => VerticesSubSection::Unnamed0E,
            0x03 => VerticesSubSection::Data(VerticesDataBuffer::import(reader, nb_vertices)?),
            0x07 => VerticesSubSection::Unnamed07,
            0x01 => VerticesSubSection::Unnamed01,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x59,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl VerticesDataBuffer {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_vertices: &u32) -> Result<VerticesDataBuffer> {
        reader.check_magic_number(&[
            0x03, 0, 0, 0, 0x4, 0, 0, 0, 0x3, 0, 0, 0, 0x20, 0, 0, 0, 0x1C, 0, 0, 0,
        ])?;
        let buffer_offset = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Start(u64::from(buffer_offset)))?;
        let mut vertices = Vec::with_capacity(*nb_vertices as usize);
        for _ in 0..*nb_vertices {
            vertices.push(Vertex::import(reader)?);
        }
        Ok(VerticesDataBuffer { vertices })
    }
}

impl Vertex {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<Vertex> {
        let position_coordinates = [
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
        ];
        reader.seek(SeekFrom::Current(0x06))?;
        let texture_coordinate_u = f16::from_bits(reader.read_le_to_u16()?);
        reader.seek(SeekFrom::Current(0x06))?;
        let texture_coordinate_v = f16::from_bits(reader.read_le_to_u16()?);
        Ok(Vertex {
            position_coordinates,
            texture_coordinates: [texture_coordinate_u, texture_coordinate_v],
        })
    }
}

impl Faces {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<Faces> {
        reader.check_magic_number(&[0x46, 0, 0, 0, 0x1C, 0, 0, 0])?;
        let nb_sub_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(0x0C))?;
        let nb_faces = reader.read_le_to_u32()?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::with_capacity(nb_sub_sections as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(FacesSubSection::import(reader, &nb_faces)?);
        }
        Ok(Faces {
            nb_faces,
            sub_sections,
        })
    }
}

impl FacesSubSection {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_faces: &u32) -> Result<FacesSubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x45 => FacesSubSection::Data(FacesDataBuffer::import(reader, nb_faces)?),
            0x6E => FacesSubSection::Unnamed6E,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x46,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl FacesDataBuffer {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_faces: &u32) -> Result<FacesDataBuffer> {
        reader.check_magic_number(&[0x45, 0, 0, 0, 0x14, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(0x0C))?;
        let mut faces = Vec::with_capacity(*nb_faces as usize);
        for _ in 0..*nb_faces {
            faces.push(Face::import(reader)?);
        }
        Ok(FacesDataBuffer { faces })
    }
}

impl Face {
    pub fn import<R: Read>(reader: &mut R) -> Result<Face> {
        Ok(Face {
            points: [
                reader.read_le_to_u16()?,
                reader.read_le_to_u16()?,
                reader.read_le_to_u16()?,
            ],
        })
    }
}
