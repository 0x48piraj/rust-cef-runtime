use rust_cef_runtime::Runtime;

mod common {
    pub mod frontend;
}

fn main() {
    let (root, url) = common::frontend::resolve("demo");

    Runtime::set_asset_root(root);

    if let Err(e) = Runtime::run(url) {
        eprintln!("Failed to start runtime: {}", e);
        std::process::exit(1);
    }
}
