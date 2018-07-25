use half::f16;
use std::io::{Read, Seek};
use Result;

/// Defines all the geometry of the model
pub struct ModelData {
    pub zero_a: Unnamed0A,
}

impl ModelData {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        strings_table: &Vec<String>,
    ) -> Result<ModelData> {
        unimplemented!();
    }
}

pub struct Unnamed0A {
    pub sub_sections: Vec<SubSections>,
}

pub enum SubSections {
    Vertices(Vertices),
    Faces(Faces),
}

pub struct Vertices {
    pub nb_vertices: u32,
    pub sub_sections: Vec<VerticesSubSections>,
}

pub enum VerticesSubSections {
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
