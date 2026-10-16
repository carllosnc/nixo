fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        res.set("FileDescription", "Nixo Media Player");
        res.set("ProductName", "Nixo Media Player");
        res.set("OriginalFilename", "nixo.exe");
        res.set(
            "LegalCopyright",
            "Copyright (c) 2026 Carlos Costa & Nixo Contributors",
        );
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resources: {e}");
        }
    }
}
