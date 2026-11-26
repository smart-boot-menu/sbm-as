//! Second image started by the probe. It prints one line and returns.

#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    let _ = uefi::boot::set_watchdog_timer(0, 0x10000, None);
    uefi::println!("SBM-AS-CHAIN: ok");
    Status::SUCCESS
}
