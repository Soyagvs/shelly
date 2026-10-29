fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--version")) {
        println!("shelly-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    shelly::run_lsp();
}
