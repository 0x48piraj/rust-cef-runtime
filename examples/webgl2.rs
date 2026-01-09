use rust_cef_runtime::Runtime;
use cef::CefString;

mod common {
    pub mod frontend;
}

fn main() {
    let url = common::frontend::resolve("webgl2");
    Runtime::run(url);

}
