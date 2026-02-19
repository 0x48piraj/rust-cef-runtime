//! app:// scheme support for local assets.

use cef::*;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

//
// SchemeHandlerFactory
//

wrap_scheme_handler_factory! {
    pub struct AppSchemeHandlerFactory;

    impl SchemeHandlerFactory {
        fn create(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _scheme_name: Option<&CefString>,
            request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {

            if let Some(req) = request {
                let url: CefString = (&req.url()).into();
                println!("SchemeHandlerFactory::create called for URL: {}", url.to_string());
            }

            Some(AppResourceHandler::new(
                Arc::new(Mutex::new(Vec::new())),
                Arc::new(Mutex::new(0)),
                Arc::new(Mutex::new(String::from("text/html"))),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicI32::new(200)),
            ))
        }
    }
}

//
// ResourceHandler
//

wrap_resource_handler! {
    pub struct AppResourceHandler {
        data: Arc<Mutex<Vec<u8>>>,
        offset: Arc<Mutex<usize>>,
        mime: Arc<Mutex<String>>,

        ready: Arc<AtomicBool>,
        failed: Arc<AtomicBool>,
        status: Arc<AtomicI32>,
    }

    impl ResourceHandler {

        fn open(
            &self,
            request: Option<&mut Request>,
            handle_request: Option<&mut i32>,
            callback: Option<&mut Callback>,
        ) -> i32 {

            // tell CEF we will handle asynchronously
            if let Some(hr) = handle_request {
                *hr = 0;
            }

            let request = request.unwrap();
            let callback = callback.unwrap().clone();

            // Convert cef string to Rust string
            let url: CefString = (&request.url()).into();
            let url = url.to_string();

            println!("ResourceHandler::open (async) {}", url);

            let data = self.data.clone();
            let offset = self.offset.clone();
            let mime = self.mime.clone();
            let ready = self.ready.clone();
            let failed = self.failed.clone();
            let status = self.status.clone();

            crate::fs_pool::spawn_io(move || {

                // Strip scheme and handle trailing slashes
                let path = url
                    .strip_prefix("app://app/")
                    .unwrap_or("index.html")
                    .trim_start_matches('/')
                    .trim_end_matches('/');

                // Handle empty path (app:// or app:///)
                let path = if path.is_empty() { "index.html" } else { path };

                println!("Resolved path: {}", path);

                // Resolve relative to CWD (set by frontend resolver)
                let root = crate::runtime::Runtime::asset_root();
                let full_path = match safe_join(&root, path) {
                    Some(p) => p,
                    None => {
                        failed.store(true, Ordering::Release);
                        status.store(404, Ordering::Release);
                        ready.store(true, Ordering::Release);
                        callback.cont();
                        return;
                    }
                };

                println!("Full file path: {:?}", full_path);

                match std::fs::read(&full_path) {
                    Ok(bytes) => {
                        *data.lock().unwrap() = bytes;
                        *offset.lock().unwrap() = 0;
                        *mime.lock().unwrap() = mime_from_path(&full_path).to_string();
                        status.store(200, Ordering::Release);
                    }
                    Err(_) => {
                        failed.store(true, Ordering::Release);
                        status.store(404, Ordering::Release);
                    }
                }

                ready.store(true, Ordering::Release);
                callback.cont();
            });

            1
        }

        fn read(
            &self,
            data_out: *mut u8,
            bytes_to_read: i32,
            bytes_read: Option<&mut i32>,
            _callback: Option<&mut ResourceReadCallback>,
        ) -> i32 {

            if !self.ready.load(Ordering::Acquire) {
                *bytes_read.unwrap() = 0;
                return 0; // wait, not EOF
            }

            if self.failed.load(Ordering::Acquire) {
                *bytes_read.unwrap() = 0;
                return 0;
            }

            let mut offset = self.offset.lock().unwrap();
            let data = self.data.lock().unwrap();

            let remaining = &data[*offset..];
            let read = remaining.len().min(bytes_to_read as usize);

            unsafe {
                std::ptr::copy_nonoverlapping(remaining.as_ptr(), data_out, read);
            }

            *offset += read;
            *bytes_read.unwrap() = read as i32;

            (read > 0) as i32
        }

        fn response_headers(
            &self,
            response: Option<&mut Response>,
            response_length: Option<&mut i64>,
            _redirect_url: Option<&mut CefString>,
        ) {
            let response = response.unwrap();

            let status = self.status.load(Ordering::Acquire);
            response.set_status(status);

            if status == 200 {
                let mime = self.mime.lock().unwrap();
                response.set_mime_type(Some(&CefString::from(mime.as_str())));
            }

            *response_length.unwrap() = self.data.lock().unwrap().len() as i64;
        }
    }

}

fn mime_from_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn safe_join(root: &Path, request: &str) -> Option<PathBuf> {
    let candidate = root.join(request);
    let canonical = candidate.canonicalize().ok()?;

    if canonical.starts_with(root) {
        Some(canonical)
    } else {
        None
    }
}
