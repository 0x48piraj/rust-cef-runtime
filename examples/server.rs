use rust_cef_runtime::App;

fn main() {
    App::new("http://localhost:8000").run_or_exit();
}
