extern crate ez_io;
extern crate magic_number;

pub mod error;

use error::ISM2ImportError;
use ez_io::ReadE;
use magic_number::check_magic_number;
use std::io::{Read, Seek, SeekFrom};

type CResult<T> = Result<T, ISM2ImportError>;

/// The main entry point of this library.
/// This represents the file at the highest level.
pub struct ISM2 {
    pub version: u32,
    pub file_size: u32,
    pub sections: Vec<Section>
}

#[repr(u32)]
pub enum Section {
    StringsTable(Vec<String>),
    JointDefinition,
    JointExtra,
    ModelData
}

impl ISM2 {
    /// Imports ISM2 from the binary file
    pub fn import<R: Read + Seek>(reader: &mut R) -> CResult<ISM2> {
        check_magic_number(reader, vec![b'I', b'S', b'M', b'2'])?;
        let version = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(8))?;
        let file_size = reader.read_le_to_u32()?;
        let nb_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(8))?;
        let mut section_offsets: Vec<(u32, u32)> = Vec::with_capacity(nb_sections as usize);  // Get the offsets for each section
        for _ in 0..nb_sections {
            let section_magic_number = reader.read_le_to_u32()?;
            let section_offset = reader.read_le_to_u32()?;
            section_offsets.push((section_magic_number, section_offset));
        }
        let mut sections = Vec::with_capacity(nb_sections as usize);  // Read each section individually
        let mut strings_table = Vec::new();  // Initialize this right away to provide something to other sections
        for section_offset in section_offsets {
            match section_offset.0 {
                0x21 => {  // Strings Table
                    strings_table = import_strings_table(reader)?;
                    sections.push(Section::StringsTable(strings_table.clone()));
                }
                0x03 => {  // Joint Definition
                    unimplemented!();
                }
                0x32 => {  // Joint Extra Information
                    unimplemented!();
                }
                0x0B => {  // Model Data
                    unimplemented!();
                }
                _ => {}
            }
        }
        Ok(ISM2 {
            version,
            file_size,
            sections
        })
    }
}

pub fn import_strings_table<R: Read + Seek>(reader: &mut R) -> CResult<Vec<String>> {
    check_magic_number(reader, vec![0x21, 0x00, 0x00, 0x00])?;
    reader.seek(SeekFrom::Current(4))?;
    let nb_entries = reader.read_le_to_u32()?;
    let mut entries_offsets = Vec::with_capacity(nb_entries as usize);
    for _ in 0..nb_entries {
        entries_offsets.push(reader.read_le_to_u32()?);
    }
    let mut strings_table = Vec::with_capacity(nb_entries as usize);
    for offset in entries_offsets {
        reader.seek(SeekFrom::Start(u64::from(offset)))?;
        let mut text = String::new();
        loop {
            match reader.read_to_u8()? {
                0x00 => {
                    strings_table.push(text);
                    break;
                }
                x => text.push(char::from(x))
            }
        }
    }
    Ok(strings_table)
}