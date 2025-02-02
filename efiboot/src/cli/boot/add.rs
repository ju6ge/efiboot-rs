//! This module hands everything related to the 'boot add' subcommand

use crate::{cli::boot::get_entries::print_var, exit_code::ExitCode};

use byteorder::{LittleEndian, ReadBytesExt};
use efivar::{
    boot::{BootEntry, BootEntryAttributes, BootVarName, BootVariable, FilePath, FilePathList},
    efi::Variable,
    VarManager,
};
use itertools::Itertools;

use super::partition;

/// get a boot entry ID that isnt used
pub fn get_used_ids(manager: &dyn VarManager) -> Vec<u16> {
    manager
        .get_all_vars()
        .unwrap()
        .filter(|var| var.vendor().is_efi())
        .filter_map(|var| var.boot_var_id())
        .collect_vec()
}

/// check if a partition+file is valid (exists), if the partition is mounted
fn check(partition: &str, file: &str) -> bool {
    if let Some(mount_point) = partition::get_mount_point(partition) {
        eprintln!(
            "Partition {} is mounted on {}. Verifying file location {file} is valid",
            partition,
            mount_point.display()
        );
        let full_path = mount_point.join(file);
        if let Ok(md) = std::fs::metadata(&full_path) {
            if md.is_file() {
                eprintln!("File location is valid");
                return true;
            }
        }
        eprintln!("{} is not a valid file", full_path.display());
        false
    } else {
        true
    }
}

/// try to fix common user errors with the file path
fn fix_file_path(mut file_path: String) -> String {
    file_path = file_path.replace('/', "\\");
    if !file_path.starts_with('\\') {
        file_path.insert(0, '\\');
    }
    file_path
}

pub fn run(
    manager: &mut dyn VarManager,
    partition: Option<String>,
    file_path: String,
    description: String,
    force: bool,
    id: Option<u16>,
) -> ExitCode {
    // get necessary information from the partition to create an entry
    let efi_partition = {
        if let Some(partition) = partition {
            // do not continue is the file has been identified as non-existant
            // ( check() has already printed the error message to the user )
            if !force && !check(&partition, &file_path) {
                return ExitCode::FAILURE;
            }
            partition::retrieve_efi_partition_data(&partition)
        } else {
            eprintln!("No partition selected. Using active boot partition");
            let active_id = manager
                .read(&Variable::new("BootCurrent"))
                .unwrap()
                .0
                .as_slice()
                .read_u16::<LittleEndian>()
                .unwrap();

            let boot_entry =
                BootEntry::read(&*manager, &Variable::new(&active_id.boot_var_name())).unwrap();

            boot_entry.file_path_list.unwrap().hard_drive
        }
    };

    let file_path_list = FilePathList {
        hard_drive: efi_partition,
        file_path: FilePath {
            path: fix_file_path(file_path).into(),
        },
    };

    // create boot entry
    let entry = BootEntry {
        attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
        description,
        file_path_list: Some(file_path_list),
        optional_data: vec![],
    };

    // assign the boot entry an id
    let id: u16 = {
        let used_boot_ids = get_used_ids(&*manager);
        if let Some(id) = id {
            if used_boot_ids.contains(&id) {
                eprintln!("Boot entry with id {id:04X} already exists. Delete it first");
                return ExitCode::FAILURE;
            }
            id
        } else {
            let id = (0x0001..0xFFFF)
                .find(|&i| !used_boot_ids.contains(&i))
                .unwrap();
            println!("Chose id {id:04X} for boot entry");
            id
        }
    };

    // create the entry
    manager.add_boot_entry(id, entry.clone()).unwrap();

    // add it to boot order
    let mut ids = manager.get_boot_order().unwrap();
    ids.insert(0, id);
    manager.set_boot_order(ids).unwrap();

    println!("Added entry with success");
    print_var(&BootVariable { entry, id }, true, 0x0000);

    ExitCode::SUCCESS
}
