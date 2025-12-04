fn main() {
    std::process::Command::new("qemu-system-x86_64")
        .arg("-bios")
        .arg(ovmf_prebuilt::ovmf_pure_efi())
        .arg("-drive")
        .arg(format!("format=raw,file={}", env!("UEFI_IMAGE")))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
