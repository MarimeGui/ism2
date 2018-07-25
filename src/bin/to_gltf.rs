extern crate clap;
extern crate ism2;

use clap::{App, Arg};
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = App::new("ISM2 to GLTF Converter")
        .version("0.1")
        .author("Marime Gui")
        .about("It converts ISM2 files to GLTF files, should work with most files")
        .arg(
            Arg::with_name("INPUT")
                .help("ISM2 file or folder to convert")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Folder name for output")
                .index(2),
        )
        .get_matches();

    let input_path = Path::new(matches.value_of("INPUT").unwrap());
    if !input_path.exists() {
        eprintln!("Error: The specified input file does not exist or is unaccessible.");
        exit(1);
    }
    let input_path_dir = if input_path.is_file() {
        input_path.parent()
    } else {
        input_path
    };

    let output_path_dir = match matches.value_of("OUTPUT") {
        Ok(output) => Path::new(output),
        Err(_) => input_path_dir.join("gltf_export/"),
    };
}
