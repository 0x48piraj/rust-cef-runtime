//! app:// scheme support for local assets.

use cef::*;
use cef::rc::*;
use std::cell::RefCell;

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
                RefCell::new(Vec::new()),
                RefCell::new(0),
                RefCell::new(CefString::from("text/html")),
            ))
        }
    }
}

//
// ResourceHandler
//

wrap_resource_handler! {
    pub struct AppResourceHandler {
        data: RefCell<Vec<u8>>,
        offset: RefCell<usize>,
        mime: RefCell<CefString>,
    }

    impl ResourceHandler {
        fn open(
            &self,
            request: Option<&mut Request>,
            handle_request: Option<&mut i32>,
            _callback: Option<&mut Callback>,
        ) -> i32 {
            if let Some(hr) = handle_request {
                *hr = 1;
            }

            let request = request.unwrap();

            // Convert cef string to Rust string
            let url: CefString = (&request.url()).into();
            let url = url.to_string();
            
            println!("ResourceHandler::open called for URL: {}", url);

            // Strip scheme and handle trailing slashes
            let path = url
                .strip_prefix("app://app/")
                .unwrap_or("index.html")
                .trim_start_matches('/')
                .trim_end_matches('/'); // Remove trailing slash

            // Handle empty path (app:// or app:///)
            let path = if path.is_empty() { "index.html" } else { path };

            println!("Resolved path: {}", path);

            // Resolve relative to CWD (set by frontend resolver)
            let full_path = std::env::current_dir()
                .unwrap()
                .join(path);

            println!("Full file path: {:?}", full_path);

            let bytes = match std::fs::read(&full_path) {
                Ok(b) => {
                    println!("Successfully read {} bytes", b.len());
                    b
                },
                Err(e) => {
                    eprintln!("Failed to read file {:?}: {}", full_path, e);
                    return 0; // Return 0 to indicate failure
                }
            };

            *self.data.borrow_mut() = bytes;
            *self.offset.borrow_mut() = 0;
            *self.mime.borrow_mut() = mime_from_path(&full_path);

            1
        }

        fn read(
            &self,
            data_out: *mut u8,
            bytes_to_read: i32,
            bytes_read: Option<&mut i32>,
            _callback: Option<&mut ResourceReadCallback>,
        ) -> i32 {
            let mut offset = self.offset.borrow_mut();
            let data = self.data.borrow();

            let remaining = &data[*offset..];
            let read = remaining.len().min(bytes_to_read as usize);

            unsafe {
                std::ptr::copy_nonoverlapping(
                    remaining.as_ptr(),
                    data_out,
                    read,
                );
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
            response.set_status(200);
            response.set_mime_type(Some(&self.mime.borrow()));
            *response_length.unwrap() = self.data.borrow().len() as i64;
        }
    }
}

fn mime_from_path(path: &std::path::Path) -> CefString {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => CefString::from("text/html"),
        Some("js") => CefString::from("text/javascript"),
        Some("css") => CefString::from("text/css"),
        Some("json") => CefString::from("application/json"),
        Some("wasm") => CefString::from("application/wasm"),
        Some("svg") => CefString::from("image/svg+xml"),
        Some("png") => CefString::from("image/png"),
        Some("jpg") | Some("jpeg") => CefString::from("image/jpeg"),
        Some("ico") => CefString::from("image/x-icon"),
        _ => CefString::from("application/octet-stream"),
    }
}