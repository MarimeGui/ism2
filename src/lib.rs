extern crate ez_io;
extern crate half;
extern crate magic_number;

pub mod error;
pub mod joint_definition;
pub mod joint_extra;
pub mod model_data;

use error::ISM2ImportError;
use ez_io::ReadE;
use joint_definition::JointDefinition;
use joint_extra::JointExtra;
use magic_number::check_magic_number;
use model_data::ModelData;
use std::io::{Read, Seek, SeekFrom};

type Result<T> = std::result::Result<T, ISM2ImportError>;

/// The main entry point of this library.
/// This represents the file at the highest level.
pub struct ISM2 {
    pub version: u32,
    pub file_size: u32,
    pub string_table: Vec<String>,
    pub sections: Vec<Section>,
}

struct SectionInfo {
    magic_number: u32,
    offset: u32,
}

/// Lists section types
pub enum Section {
    JointDefinition(JointDefinition),
    JointExtra(JointExtra),
    ModelData(ModelData),
}

impl ISM2 {
    /// Imports ISM2 from the binary file
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<ISM2> {
        // Hello There! General Information
        check_magic_number(reader, vec![b'I', b'S', b'M', b'2'])?;
        let version = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(8))?;
        let file_size = reader.read_le_to_u32()?;
        let nb_sections = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(8))?;
        // Offsets to sections
        let mut section_offsets: Vec<SectionInfo> = Vec::with_capacity(nb_sections as usize); // Get the offsets for each section
        for _ in 0..nb_sections {
            section_offsets.push(SectionInfo::import(reader)?);
        }
        // Read the string table, incorporating it in the main TID struct for convenience
        let string_table: Vec<String> = {
            match section_offsets.get(0) {
                Some(o) => {
                    match o.magic_number {
                        0x21 => {
                            reader.seek(SeekFrom::Start(u64::from(o.offset)))?;
                            import_strings_table(reader)?
                        }
                        _ => panic!(), // TODO Create relevant error
                    }
                }
                None => panic!(), // TODO Create relevant error
            }
        };
        // Read all other sections
        let mut sections = Vec::with_capacity(nb_sections as usize);
        for section_info in section_offsets.into_iter().skip(1) {
            // Skip the Strings Table, already handled above
            reader.seek(SeekFrom::Start(u64::from(section_info.offset)))?;
            match section_info.magic_number {
                0x03 => {
                    // Joint Definition
                    sections.push(Section::JointDefinition(JointDefinition::import(
                        reader,
                        &string_table,
                    )?));
                }
                0x32 => {
                    // Joint Extra Information
                    sections.push(Section::JointExtra(JointExtra::import(
                        reader,
                        &string_table,
                    )?));
                }
                0x0B => {
                    // Model Data
                    sections.push(Section::ModelData(ModelData::import(
                        reader,
                        &string_table,
                    )?));
                }
                _ => {}
            }
        }
        Ok(ISM2 {
            version,
            file_size,
            string_table,
            sections,
        })
    }
}

impl SectionInfo {
    fn import<R: Read>(reader: &mut R) -> Result<SectionInfo> {
        Ok(SectionInfo {
            magic_number: reader.read_le_to_u32().unwrap(),
            offset: reader.read_le_to_u32().unwrap(),
        })
    }
}

/// Reads a String Table from a file and returns a vector containing all entries, preserving the original indices.
pub fn import_strings_table<R: Read + Seek>(reader: &mut R) -> Result<Vec<String>> {
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
                x => text.push(char::from(x)),
            }
        }
    }
    Ok(strings_table)
}
