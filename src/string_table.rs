use ez_io::{MagicNumberCheck, ReadE};
use std::io::{Read, Seek, SeekFrom};
use Result;

/// Reads a String Table from a file and returns a vector containing all entries, preserving the original indices.
pub fn import_strings_table<R: Read + Seek>(reader: &mut R) -> Result<Vec<String>> {
    reader.check_magic_number(&[0x21, 0x00, 0x00, 0x00])?;
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
