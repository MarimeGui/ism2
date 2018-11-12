use ez_io::{MagicNumberCheck, ReadE};
use std::io::{Read, Seek, SeekFrom};
use crate::Result;

pub struct TextureDefinition {
    pub sub_sections: Vec<Texture>,
}

pub struct Texture {
    pub base_name: String,
    pub original_location: String,
    pub original_name: String,
}

impl TextureDefinition {
    pub fn import<R: Read + Seek>(
        reader: &mut R,
        string_table: &[String],
    ) -> Result<TextureDefinition> {
        reader.check_magic_number(&[0x2E, 0, 0, 0])?;
        reader.seek(SeekFrom::Current(4))?;
        let nb_sub_sections = reader.read_le_to_u32()?;
        let mut offsets = Vec::with_capacity(nb_sub_sections as usize);
        for _ in 0..nb_sub_sections {
            offsets.push(reader.read_le_to_u32()?);
        }
        let mut sub_sections = Vec::new();
        for o in offsets {
            reader.seek(SeekFrom::Start(u64::from(o)))?;
            sub_sections.push(Texture::import(reader, string_table)?);
        }
        Ok(TextureDefinition { sub_sections })
    }
}

impl Texture {
    pub fn import<R: Read + Seek>(reader: &mut R, string_table: &[String]) -> Result<Texture> {
        reader.check_magic_number(&[0x2D, 0, 0, 0])?;
        let a = reader.read_le_to_u32()?;
        let _ = reader.read_le_to_u32()?;
        let b = reader.read_le_to_u32()?;
        let c = reader.read_le_to_u32()?;
        Ok(Texture {
            base_name: string_table[a as usize].clone(),
            original_location: string_table[b as usize].clone(),
            original_name: string_table[c as usize].clone(),
        })
    }
}
