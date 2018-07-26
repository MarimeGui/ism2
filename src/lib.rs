extern crate ez_io;
extern crate half;

pub mod error;
pub mod joint_definition;
pub mod joint_extra;
pub mod model_data;
pub mod string_table;

use error::ISM2ImportError;
use ez_io::{MagicNumberCheck, ReadE};
use joint_definition::JointDefinition;
use joint_extra::JointExtra;
use model_data::ModelData;
use std::io::{Read, Seek, SeekFrom};
use string_table::import_strings_table;

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
        reader.check_magic_number(&[b'I', b'S', b'M', b'2'])?;
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
                    // sections.push(Section::JointExtra(JointExtra::import(
                    //     reader,
                    //     &string_table,
                    // )?));
                }
                0x0B => {
                    // Model Data
                    sections.push(Section::ModelData(ModelData::import(reader)?));
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
