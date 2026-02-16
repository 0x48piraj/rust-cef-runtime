use rust_cef_runtime::Runtime;
use cef::CefString;

mod common {
    pub mod frontend;
}

fn main() {
    let (root, url) = common::frontend::resolve("files-cors");

    Runtime::set_asset_root(root);
    Runtime::run(url);
}
