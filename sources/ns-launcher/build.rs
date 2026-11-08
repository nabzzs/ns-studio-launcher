fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../branding/ns-studio-logo.ico")
        .set("ProductName", "NS Studio")
        .set("FileDescription", "NS Studio creative suite launcher")
        .set("OriginalFilename", "NS-Studio.exe");
    if let Err(e) = res.compile() {
        println!("cargo:warning=NS launcher resources: {e}");
    }
}
