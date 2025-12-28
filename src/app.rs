//! Root CEF application object.

use cef::*;
use cef::rc::*;
use std::sync::{Arc, Mutex};

use crate::browser::DemoBrowserProcessHandler;

wrap_app! {
    pub struct DemoApp {
        window: Arc<Mutex<Option<Window>>>,
        start_url: CefString,
    }

    impl App {
        fn on_before_command_line_processing(
            &self,
            _process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            if let Some(command_line) = command_line {
                // Allow loading local files (file://) and disable CORS for development
                command_line.append_switch(Some(&CefString::from("allow-file-access-from-files")));
                command_line.append_switch(Some(&CefString::from("disable-web-security")));
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(
                DemoBrowserProcessHandler::new(
                    self.window.clone(),
                    self.start_url.clone(),
                )
            )
        }
    }
}
