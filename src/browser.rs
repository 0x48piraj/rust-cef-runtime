//! Browser-process lifecycle handling.

use cef::*;
use cef::rc::*;
use std::sync::{Arc, Mutex};

use crate::{client::DemoClient, window::DemoWindowDelegate};

wrap_browser_process_handler! {
    pub struct DemoBrowserProcessHandler {
        window: Arc<Mutex<Option<Window>>>,
        start_url: CefString,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            println!("on_context_initialized called");
            println!("Registering scheme handler factory for app://");
            
            // Register the scheme handler factory for app:// URLs
            let result = register_scheme_handler_factory(
                Some(&CefString::from("app")),
                Some(&CefString::from("app")),
                Some(&mut crate::scheme::AppSchemeHandlerFactory::new()),
            );

            println!("register_scheme_handler_factory result: {}", result);

            let mut client = DemoClient::new();
            let url = self.start_url.clone();
            
            println!("Creating browser with URL: {}", url.to_string());

            let browser_view = browser_view_create(
                Some(&mut client),
                Some(&url),
                Some(&Default::default()),
                None,
                None,
                None,
            )
            .expect("browser_view_create failed");

            let mut delegate = DemoWindowDelegate::new(browser_view);

            if let Ok(mut window) = self.window.lock() {
                *window = Some(
                    window_create_top_level(Some(&mut delegate))
                        .expect("window_create_top_level failed"),
                );
            }
        }
    }
}
