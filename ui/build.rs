fn main() {
    println!("cargo:rerun-if-changed=ui/lumen.slint");

    slint_build::compile("ui/Shell.slint").expect("Failed to compile Slint UI");

    let mut res = winres::WindowsResource::new();
    res.set_icon("../assets/lumen.ico");
    res.set("FileDescription", "Lumen");
    res.set("ProductName", "Lumen");
    res.set("InternalName", "Lumen");
    res.compile().unwrap();
}
