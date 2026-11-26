//! Protocol probe for U-Boot's EFI loader.
//!
//! This is not a boot menu. It records whether the firmware provides the
//! protocols Smart Boot Menu needs, writes the ESP policy file, and starts
//! one other EFI image.
//!
//! ESP policy path: \EFI\sbm\boot-policy.json
//! Chain target path: \EFI\sbm\chain-target.efi

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use uefi::boot::LoadImageSource;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::console::pointer::{AbsolutePointer, Pointer};
use uefi::proto::console::text::{Input, InputEx};
use uefi::proto::media::block::BlockIO;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::runtime::{self, VariableAttributes, VariableVendor};
use uefi::{Guid, guid};

const FDT_TABLE: Guid = guid!("b1b621d5-f19c-41a5-830b-d9152c69aae0");
const PROBE_VENDOR: Guid = guid!("d8f4a21c-7b6e-4c91-9e32-8a5c1f0e3b40");
const POLICY: &uefi::CStr16 = cstr16!("\\EFI\\sbm\\boot-policy.json");
const CHAIN: &uefi::CStr16 = cstr16!("\\EFI\\sbm\\chain-target.efi");
const EXTERNAL: &uefi::CStr16 = cstr16!("\\EFI\\sbm\\external-payload.efi");
const MAX_IMAGE: usize = 8 * 1024 * 1024;
const POLICY_BYTES: &[u8] = br#"{"schema":"sbm-as-boot-policy","version":1,"store":"esp-file"}"#;

#[entry]
fn main() -> Status {
    let _ = uefi::boot::set_watchdog_timer(0, 0, None);
    uefi::println!("SBM-AS-PROBE: start");
    uefi::println!("SBM-AS-PROBE: qemu-virt-is-not-apple-silicon");
    report_gop();
    report_input();
    report_block_io();
    report_fdt();
    report_variable();
    discover_and_chainload();
    uefi::println!("SBM-AS-PROBE: done");
    Status::SUCCESS
}

fn report_gop() {
    let Ok(handle) = uefi::boot::get_handle_for_protocol::<GraphicsOutput>() else {
        uefi::println!("SBM-AS-PROBE: gop=absent");
        return;
    };
    let Ok(mut gop) = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(handle) else {
        uefi::println!("SBM-AS-PROBE: gop=present-unopened");
        return;
    };
    let (width, height) = gop.current_mode_info().resolution();
    let filled = gop
        .blt(BltOp::VideoFill {
            color: BltPixel::new(255, 255, 255),
            dest: (0, 0),
            dims: (32, 32),
        })
        .is_ok();
    uefi::println!("SBM-AS-PROBE: gop={width}x{height} filled={filled}");
}

fn report_input() {
    let keyboard = uefi::boot::get_handle_for_protocol::<Input>().is_ok()
        || uefi::boot::get_handle_for_protocol::<InputEx>().is_ok();
    let pointer = uefi::boot::get_handle_for_protocol::<Pointer>().is_ok()
        || uefi::boot::get_handle_for_protocol::<AbsolutePointer>().is_ok();
    uefi::println!("SBM-AS-PROBE: keyboard={keyboard}");
    uefi::println!("SBM-AS-PROBE: pointer={pointer}");
}

fn report_block_io() {
    let count = uefi::boot::find_handles::<BlockIO>()
        .map(|handles| handles.len())
        .unwrap_or(0);
    uefi::println!("SBM-AS-PROBE: block-io={count}");
}

fn report_fdt() {
    let present = uefi::system::with_config_table(|tables| {
        tables.iter().any(|table| table.guid == FDT_TABLE)
    });
    uefi::println!("SBM-AS-PROBE: fdt={present}");
}

fn report_variable() {
    let vendor = VariableVendor(PROBE_VENDOR);
    let name = cstr16!("SbmAsProbe");
    let volatile = VariableAttributes::BOOTSERVICE_ACCESS | VariableAttributes::RUNTIME_ACCESS;
    let nonvolatile = volatile | VariableAttributes::NON_VOLATILE;
    let wrote_nonvolatile = runtime::set_variable(name, &vendor, nonvolatile, b"1").is_ok();
    let wrote_volatile = if wrote_nonvolatile {
        false
    } else {
        runtime::set_variable(name, &vendor, volatile, b"1").is_ok()
    };
    let mut buffer = [0u8; 4];
    let read_back = runtime::get_variable(name, &vendor, &mut buffer).is_ok();
    uefi::println!(
        "SBM-AS-PROBE: variable=nonvolatile:{wrote_nonvolatile},volatile:{wrote_volatile},read:{read_back}"
    );
}

fn discover_and_chainload() {
    let image = uefi::boot::image_handle();
    if let Ok(mut fs) = uefi::boot::get_image_file_system(image) {
        write_policy(&mut fs);
        if let Some(bytes) = read_file(&mut fs, CHAIN) {
            uefi::println!("SBM-AS-PROBE: discovered=\\EFI\\sbm\\chain-target.efi");
            chainload(&bytes);
        } else {
            uefi::println!("SBM-AS-PROBE: chainload=missing");
        }
    } else {
        uefi::println!("SBM-AS-PROBE: image-filesystem=absent");
    }

    let Ok(handles) = uefi::boot::find_handles::<SimpleFileSystem>() else {
        return;
    };
    for handle in handles {
        let Ok(mut fs) = uefi::boot::open_protocol_exclusive::<SimpleFileSystem>(handle) else {
            continue;
        };
        if read_file(&mut fs, EXTERNAL).is_some() {
            uefi::println!("SBM-AS-PROBE: discovered=\\EFI\\sbm\\external-payload.efi");
        }
    }
}

fn write_policy(fs: &mut uefi::boot::ScopedProtocol<SimpleFileSystem>) {
    let Ok(mut root) = fs.open_volume() else {
        uefi::println!("SBM-AS-PROBE: policy-failed volume");
        return;
    };
    let _ = root.open(
        cstr16!("\\EFI"),
        FileMode::CreateReadWrite,
        FileAttribute::DIRECTORY,
    );
    let _ = root.open(
        cstr16!("\\EFI\\sbm"),
        FileMode::CreateReadWrite,
        FileAttribute::DIRECTORY,
    );
    let Ok(handle) = root.open(POLICY, FileMode::CreateReadWrite, FileAttribute::empty()) else {
        uefi::println!("SBM-AS-PROBE: policy-failed open");
        return;
    };
    let Ok(FileType::Regular(mut file)) = handle.into_type() else {
        uefi::println!("SBM-AS-PROBE: policy-failed type");
        return;
    };
    if file.write(POLICY_BYTES).is_ok() {
        uefi::println!("SBM-AS-PROBE: policy-written");
    } else {
        uefi::println!("SBM-AS-PROBE: policy-failed write");
    }
}

fn read_file(
    fs: &mut uefi::boot::ScopedProtocol<SimpleFileSystem>,
    path: &uefi::CStr16,
) -> Option<Vec<u8>> {
    let mut root = fs.open_volume().ok()?;
    let handle = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .ok()?;
    let FileType::Regular(mut file) = handle.into_type().ok()? else {
        return None;
    };
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = file.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > MAX_IMAGE {
            return None;
        }
    }
    Some(bytes)
}

fn chainload(bytes: &[u8]) {
    let loaded = uefi::boot::load_image(
        uefi::boot::image_handle(),
        LoadImageSource::FromBuffer {
            buffer: bytes,
            file_path: None,
        },
    );
    match loaded {
        Ok(handle) => match uefi::boot::start_image(handle) {
            Ok(()) => uefi::println!("SBM-AS-PROBE: chainload=ok"),
            Err(error) => uefi::println!("SBM-AS-PROBE: chainload=start-failed {error:?}"),
        },
        Err(error) => uefi::println!("SBM-AS-PROBE: chainload=load-failed {error:?}"),
    }
}
