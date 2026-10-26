// Windows の exe にアイコン(assets/icon/icon.ico)を埋め込む。失敗してもビルドは続ける。
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon/icon.ico");
        match res.compile() {
            // 静的ライブラリのままだと中身が参照されず捨てられるので、オブジェクトを直接リンクする
            Ok(()) => println!("cargo:rustc-link-arg-bins={}/resource.o", std::env::var("OUT_DIR").unwrap_or_default()),
            Err(e) => println!("cargo:warning=exe icon not embedded: {}", e),
        }
    }
    println!("cargo:rerun-if-changed=assets/icon/icon.ico");
}
