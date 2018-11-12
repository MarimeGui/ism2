use crate::error::{ISM2ImportError, UnknownSubSection};
use ez_io::{MagicNumberCheck, ReadE};
use half::f16;
use std::io::{Read, Seek, SeekFrom};
use crate::Result;

/// Defines all the geometry of the model
pub struct ModelData {
    pub zero_a: Unnamed0A,
}

pub struct Unnamed0A {
    pub sub_sections: Vec<SubSection>,
}

pub enum SubSection {
    Vertices(Vertices),
    Mesh(Mesh),
    Unnamed6E,
}

pub struct Vertices {
    pub nb_vertices: u32,
    pub attributes: Vec<VertexAttribute>,
    pub buffer: VerticesDataBuffer,
}

#[derive(Clone)]
pub struct VertexAttribute {
    pub attribute_type: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u32,
    pub buffer_offset: u32,
}

pub enum VerticesDataBuffer {
    Geometry(VerticesGeometryBuffer),
    Rigging(VerticesRiggingBuffer),
}

pub struct VerticesGeometryBuffer {
    pub vertices: Vec<VertexGeometry>,
}

pub struct VerticesRiggingBuffer {
    pub vertices: Vec<VertexRigging>,
}

pub struct Vector3D<T> {
    // Move to Utils
    pub x: T,
    pub y: T,
    pub z: T,
}

pub struct Vector2D<T> {
    // Move to Utils
    pub u: T,
    pub v: T,
}

pub struct VertexGeometry {
    pub position_coordinates: Vector3D<f32>,
    pub texture_coordinates: Vector2D<f16>,
    pub frenet_frame: FrenetFrame<f16>,
}

pub struct VertexRigging {
    pub joints: (u8, u8, u8, u8),
    pub weights: (f32, f32, f32, f32),
}

pub struct FrenetFrame<T> {
    // Move to Utils
    pub normal: Vector3D<T>,
    pub tangent: Vector3D<T>,
}

pub struct Mesh {
    pub nb_faces: u32,
    pub sub_sections: Vec<MeshSubSection>,
}

pub enum MeshSubSection {
    Faces(Faces),
    Unnamed6E,
}

pub struct Faces {
    pub faces: Vec<Face>,
}

pub struct Face {
    pub points: (u16, u16, u16),
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
            0x46 => SubSection::Mesh(Mesh::import(reader)?),
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
        let mut attributes = Vec::with_capacity(nb_sub_sections as usize);
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            attributes.push(VertexAttribute::import(reader)?);
        }
        let just_for_you_special_snowflake = attributes.clone();
        let any_attribute = match just_for_you_special_snowflake.get(0) {
            Some(a) => a,
            None => return Err(ISM2ImportError::NoAttributes),
        };
        reader.seek(SeekFrom::Start(u64::from(any_attribute.buffer_offset)))?;
        let vertices_data_buffer = match any_attribute.attribute_type {
            0x00 | 0x02 | 0x0E | 0x03 => {
                VerticesDataBuffer::Geometry(VerticesGeometryBuffer::import(reader, nb_vertices)?)
            }
            0x07 | 0x01 => {
                VerticesDataBuffer::Rigging(VerticesRiggingBuffer::import(reader, nb_vertices)?)
            }
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x59,
                    magic_number_sub_section: x,
                }))
            }
        };
        Ok(Vertices {
            nb_vertices,
            attributes,
            buffer: vertices_data_buffer,
        })
    }
}

impl VertexAttribute {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<VertexAttribute> {
        Ok(VertexAttribute {
            attribute_type: reader.read_le_to_u32()?,
            unknown2: reader.read_le_to_u32()?,
            unknown3: reader.read_le_to_u32()?,
            unknown4: reader.read_le_to_u32()?,
            unknown5: reader.read_le_to_u32()?,
            buffer_offset: reader.read_le_to_u32()?,
        })
    }
}

impl VerticesGeometryBuffer {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        nb_vertices: u32,
    ) -> Result<VerticesGeometryBuffer> {
        let mut vertices = Vec::with_capacity(nb_vertices as usize);
        for _ in 0..nb_vertices {
            vertices.push(VertexGeometry::import(reader)?);
        }
        Ok(VerticesGeometryBuffer { vertices })
    }
}

impl VerticesRiggingBuffer {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        nb_vertices: u32,
    ) -> Result<VerticesRiggingBuffer> {
        let mut vertices = Vec::with_capacity(nb_vertices as usize);
        for _ in 0..nb_vertices {
            vertices.push(VertexRigging::import(reader)?);
        }
        Ok(VerticesRiggingBuffer { vertices })
    }
}

impl VertexGeometry {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<VertexGeometry> {
        let position = (
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
        );
        let normal = (
            f16::from_bits(reader.read_le_to_u16()?),
            f16::from_bits(reader.read_le_to_u16()?),
            f16::from_bits(reader.read_le_to_u16()?),
        );
        let texture_coordinate_u = f16::from_bits(reader.read_le_to_u16()?);
        let tangent = (
            f16::from_bits(reader.read_le_to_u16()?),
            f16::from_bits(reader.read_le_to_u16()?),
            f16::from_bits(reader.read_le_to_u16()?),
        );
        let texture_coordinate_v = f16::from_bits(reader.read_le_to_u16()?);
        reader.seek(SeekFrom::Current(0x04))?;
        Ok(VertexGeometry {
            position_coordinates: Vector3D {
                x: position.0,
                y: position.1,
                z: position.2,
            },
            texture_coordinates: Vector2D {
                u: texture_coordinate_u,
                v: texture_coordinate_v,
            },
            frenet_frame: FrenetFrame {
                normal: Vector3D {
                    x: normal.0,
                    y: normal.1,
                    z: normal.2,
                },
                tangent: Vector3D {
                    x: tangent.0,
                    y: tangent.1,
                    z: tangent.2,
                },
            },
        })
    }
}

impl VertexRigging {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<VertexRigging> {
        let joints = (
            reader.read_to_u8()?,
            reader.read_to_u8()?,
            reader.read_to_u8()?,
            reader.read_to_u8()?,
        );
        let weights = (
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
            reader.read_le_to_f32()?,
        );
        reader.seek(SeekFrom::Current(12))?;
        Ok(VertexRigging { joints, weights })
    }
}

impl Mesh {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<Mesh> {
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
            sub_sections.push(MeshSubSection::import(reader, nb_faces)?);
        }
        Ok(Mesh {
            nb_faces,
            sub_sections,
        })
    }
}

impl MeshSubSection {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_faces: u32) -> Result<MeshSubSection> {
        let magic_number = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(-4))?;
        Ok(match magic_number {
            0x45 => MeshSubSection::Faces(Faces::import(reader, nb_faces)?),
            0x6E => MeshSubSection::Unnamed6E,
            x => {
                return Err(ISM2ImportError::UnknownSubSection(UnknownSubSection {
                    magic_number_section: 0x46,
                    magic_number_sub_section: x,
                }))
            }
        })
    }
}

impl Faces {
    pub fn import<R: Read + Seek>(reader: &mut R, nb_faces: u32) -> Result<Faces> {
        reader.check_magic_number(&[0x45, 0, 0, 0, 0x14, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(0x0C))?;
        let mut faces = Vec::with_capacity(nb_faces as usize);
        for _ in 0..nb_faces {
            faces.push(Face::import(reader)?);
        }
        Ok(Faces { faces })
    }
}

impl Face {
    pub fn import<R: Read>(reader: &mut R) -> Result<Face> {
        Ok(Face {
            points: (
                reader.read_le_to_u16()?,
                reader.read_le_to_u16()?,
                reader.read_le_to_u16()?,
            ),
        })
    }
}
