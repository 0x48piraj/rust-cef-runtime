use rust_cef_runtime::Runtime;

mod common {
    pub mod frontend;
}

fn main() {
    let url = common::frontend::resolve("dom");
    Runtime::run(url);
}
