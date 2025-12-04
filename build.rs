use std::path::PathBuf;

fn main() {
    let kernel = std::env::var("CARGO_BIN_FILE_SAILOR_KERNEL")
        .map(PathBuf::from)
        .unwrap();
    let uefi = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap()
        .join("uefi.img");

    println!("{}", kernel.display());
    println!("{}", uefi.display());

    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi)
        .unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi.display());
}
