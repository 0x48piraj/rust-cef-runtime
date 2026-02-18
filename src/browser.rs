//! Browser-process lifecycle handling.
//! A BrowserProcessHandler exists per request context.
//! We only want one native window per application,
//! so we guard creation using the shared window handle.

use cef::*;
use cef::rc::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::client::DemoClient;

wrap_browser_process_handler! {
    pub struct DemoBrowserProcessHandler {
        window: Arc<Mutex<Option<Window>>>,
        start_url: CefString,

        // Keep factory alive for browser lifetime; RefCell for interior mutability
        scheme_factory: RefCell<Option<SchemeHandlerFactory>>,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            println!("on_context_initialized called");

            // Initialize IPC dispatcher
            crate::ipc_browser::init_dispatcher();
            println!("IPC dispatcher initialized");

            // Register once per request context
            if self.scheme_factory.borrow().is_none() {
                println!("Registering scheme handler factory for app://");

                // create factory (temporary mutable)
                let mut factory = crate::scheme::AppSchemeHandlerFactory::new();

                // Register the scheme handler factory for app:// URLs
                let result = register_scheme_handler_factory(
                    Some(&CefString::from("app")),
                    Some(&CefString::from("app")),
                    Some(&mut factory),
                );

                // store so CEF never calls freed memory
                *self.scheme_factory.borrow_mut() = Some(factory);

                println!("register_scheme_handler_factory result: {}", result);
            }

            // Only create window if it does not already exist
            {
                let guard = self.window.lock().unwrap();
                if guard.is_some() {
                    println!("Secondary request context; skipping window creation");
                    return;
                }
            }

            let mut client = DemoClient::new();
            let url = self.start_url.clone();

            println!("Creating main browser with URL: {}", url.to_string());

            let browser_view = browser_view_create(
                Some(&mut client),
                Some(&url),
                Some(&Default::default()),
                None,
                None,
                None,
            )
            .expect("browser_view_create failed");

            // Create delegate
            let mut delegate = crate::window::DemoWindowDelegate::new(browser_view, self.window.clone());

            // Create window
            let window = window_create_top_level(Some(&mut delegate))
                .expect("window_create_top_level failed");

            *self.window.lock().unwrap() = Some(window);
        }
    }
}
