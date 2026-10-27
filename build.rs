fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    println!("cargo:rerun-if-changed=assets/DiskAgent.ico");

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/DiskAgent.ico");
    res.set("ProductName", "DiskAgent");
    res.set("FileDescription", "DiskAgent - Disk Usage Analyzer");
    res.set("OriginalFilename", "DiskAgent.exe");
    res.set("LegalCopyright", "Copyright (C) 2026 Dimas Yuda Pratama");

    if let Err(err) = res.compile() {
        panic!("failed to compile Windows resources: {err}");
    }
}
