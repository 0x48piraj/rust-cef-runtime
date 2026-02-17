use rust_cef_runtime::{Runtime, register_command};

mod common {
    pub mod frontend;
}

fn main() {
    let (root, url) = common::frontend::resolve("demo");

    Runtime::set_asset_root(root);

    // Register commands before Runtime::run()
    register_command("echo", |payload| {
        Ok(format!("Echo: {}", payload))
    });

    if let Err(e) = Runtime::run(url) {
        eprintln!("Failed to start runtime: {}", e);
        std::process::exit(1);
    }
}
