use std::{fs::File, io::Read, path::Path};

use uuid::Uuid;

use byteorder::{LittleEndian, ReadBytesExt};

use efivar::{
    efi::{Variable, VariableFlags, VariableVendor},
    VarManager,
};

use crate::exit_code::ExitCode;

fn read_var_from_file_or_stdin(
    input_path: &Path,
) -> Result<(VariableFlags, Vec<u8>), std::io::Error> {
    if input_path.to_str().unwrap() == "-" {
        read_var_from_stdin()
    } else {
        read_var_from_file(input_path)
    }
}

fn read_var_from_stdin() -> Result<(VariableFlags, Vec<u8>), std::io::Error> {
    let mut buf: Vec<u8> = vec![];
    std::io::stdin().read_to_end(&mut buf).unwrap();

    Ok((VariableFlags::default(), buf))
}

fn read_var_from_file(input_path: &Path) -> Result<(VariableFlags, Vec<u8>), std::io::Error> {
    let mut file = File::open(input_path)?;

    let flags = VariableFlags::from_bits(file.read_u32::<LittleEndian>()?).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    Ok((flags, data))
}

pub fn run(
    manager: &mut dyn VarManager,
    input_path: &Path,
    name: &str,
    namespace: Option<Uuid>,
) -> ExitCode {
    let var = Variable::new_with_vendor(
        name,
        namespace.map_or(VariableVendor::Efi, VariableVendor::Custom),
    );

    let (flags, data) = match read_var_from_file_or_stdin(input_path) {
        Ok(inner) => inner,
        Err(err) => {
            eprintln!("Failed to read variable {}: {}", input_path.display(), err);
            return ExitCode::FAILURE;
        }
    };

    match manager.write(&var, flags, &data) {
        Ok(()) => {
            println!("Imported variable {} with success", var);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Failed to write variable {}: {}", var, err);
            ExitCode::FAILURE
        }
    }
}
