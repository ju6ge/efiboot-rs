use std::str::FromStr;

use uuid::Uuid;

use crate::boot::{BootEntry, BootEntryAttributes, EFIHardDrive, FilePath, FilePathList};

#[test]
fn dump() {
    //! Dump a BootEntry object to bytes

    let entry = BootEntry {
        attributes: BootEntryAttributes::LOAD_OPTION_ACTIVE,
        description: "GRUB".to_owned(),
        file_path_list: Some(FilePathList {
            file_path: FilePath {
                path: "\\EFI\\arch\\grubx64.efi".into(),
            },
            hard_drive: EFIHardDrive {
                partition_number: 1,
                partition_start: 10240,
                partition_size: 991232,
                partition_sig: Uuid::from_str("0ac986f5-a335-f2a8-4943-3ef0837cdce3").unwrap(),
                format: 2,
                sig_type: crate::boot::EFIHardDriveType::Gpt,
            },
        }),
        optional_data: vec![],
    };

    // this data as simply been copied from BootEntry::to_bytes(), I have not thoroughly verified if its right
    let data: [u8; 0x6E] = [
        0x01, 0x00, 0x00, 0x00, 0x5E, 0x00, 0x47, 0x00, 0x52, 0x00, 0x55, 0x00, 0x42, 0x00, 0x00,
        0x00, 0x04, 0x01, 0x2A, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x20, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0xC9, 0x86, 0xF5, 0xA3,
        0x35, 0xF2, 0xA8, 0x49, 0x43, 0x3E, 0xF0, 0x83, 0x7C, 0xDC, 0xE3, 0x02, 0x02, 0x04, 0x04,
        0x30, 0x00, 0x5C, 0x00, 0x45, 0x00, 0x46, 0x00, 0x49, 0x00, 0x5C, 0x00, 0x61, 0x00, 0x72,
        0x00, 0x63, 0x00, 0x68, 0x00, 0x5C, 0x00, 0x67, 0x00, 0x72, 0x00, 0x75, 0x00, 0x62, 0x00,
        0x78, 0x00, 0x36, 0x00, 0x34, 0x00, 0x2E, 0x00, 0x65, 0x00, 0x66, 0x00, 0x69, 0x00, 0x00,
        0x00, 0x7F, 0xFF, 0x04, 0x00,
    ];

    assert_eq!(entry.to_bytes(), data);
}
