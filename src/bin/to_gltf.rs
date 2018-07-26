extern crate clap;
extern crate ism2;

use clap::{App, Arg};
use ism2::ISM2;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = App::new("ISM2 to GLTF Converter")
        .version("0.1")
        .author("Marime Gui")
        .about("It converts ISM2 files to GLTF files, should work with most files")
        .arg(
            Arg::with_name("INPUT")
                .help("ISM2 file to convert")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Folder name for output")
                .required(true)
                .index(2),
        )
        .get_matches();

    let input_str = matches.value_of("INPUT").unwrap();
    let output_str = matches.value_of("OUTPUT").unwrap();
    let input_path = Path::new(input_str);
    let output_path = Path::new(output_str);
    if !input_path.exists() {
        eprintln!("Error: The specified input file does not exist or is unaccessible.");
        exit(1);
    }
    create_dir_all(output_path).unwrap();

    let reader = &mut BufReader::new(File::open(input_path).unwrap());

    let ism = ISM2::import(reader).unwrap();

    println!("{} sub-sections", ism.sections.len());
}
