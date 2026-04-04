use args::Args;
use assembler::{MyAssembler, writer::Writer};
use std::{
    fs::File,
    io::{BufReader, Read},
    process,
};

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(err) => {
            println!("Failed to parse arguments:\n\t{}", err);
            process::exit(1);
        }
    };
    let input_filepath = match args.input_filepath {
        Some(filepath) => {
            let filename = filepath.split('/').last().unwrap();
            if filename.split('.').last().unwrap() == "asm" {
                filepath
            } else {
                println!("Assembler only accepts .asm files");
                process::exit(1);
            }
        }
        None => {
            println!("Usage: assembler <filename.asm> [--debug] [--pretty] [--log=<console|file>]");
            process::exit(1);
        }
    };
    let mut assembler = match MyAssembler::new() {
        Ok(assembler) => assembler,
        Err(err) => {
            println!("Failed to create assembler:\n\t{}", err);
            process::exit(1);
        }
    };
    if args.debug {
        println!("Debug mode enabled.");
        if args.pretty {
            println!("ASCII binary would be prettified.");
        }
    }

    let file = match File::open(&input_filepath) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open file:\n\t{}", err);
            process::exit(1);
        }
    };

    println!("Assembly file: {}", input_filepath);
    let mut assembly_program = String::new();
    let mut reader = BufReader::new(file);
    reader
        .read_to_string(&mut assembly_program)
        .expect("Failed to read file");

    match assembler.assemble(assembly_program.as_str()) {
        Ok((binary, mut delimiter_table)) => {
            match Writer::new(args.debug, args.pretty, args.output_filepath, None) {
                Ok(mut writer) => match writer.write(binary, &mut delimiter_table) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Failed to write:\n\t{}", err);
                        process::exit(1);
                    }
                },
                Err(err) => {
                    println!("Failed to create writer:\n\t{}", err);
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            println!("Failed to assemble:\n{}", err);
            process::exit(1);
        }
    };
}
