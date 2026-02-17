use rust_cef_runtime::{Runtime, register_command};
use serde_json::{Value, json};

mod common {
    pub mod frontend;
}

fn main() {
    let (root, url) = common::frontend::resolve("demo");

    Runtime::set_asset_root(root);

    // Register commands before Runtime::run()
    register_command("echo", |v: Value| {
        Ok(v)
    });

    register_command("add", |v: Value| {
        let a = v["a"].as_i64().unwrap_or(0);
        let b = v["b"].as_i64().unwrap_or(0);
        Ok(json!(a + b))
    });

    if let Err(e) = Runtime::run(url) {
        eprintln!("Failed to start runtime: {}", e);
        std::process::exit(1);
    }
}
