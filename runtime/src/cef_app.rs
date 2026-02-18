//! Root CEF application object.

use cef::*;
use cef::rc::*;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

use crate::browser::DemoBrowserProcessHandler;
use crate::ipc_renderer::IpcRenderProcessHandler;

use cef::sys::cef_scheme_options_t::*;

wrap_app! {
    pub struct DemoApp {
        window: Arc<Mutex<Option<Window>>>,
        start_url: CefString,
    }

    impl App {
        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>,
            command_line: Option<&mut CommandLine>,
        ) {
            if process_type.is_some() {
                // Only configure the main browser process
                return;
            }

            let Some(cmd) = command_line else { return };

            // Configure GPU and rendering backend per platform.
            // Uses native compositing paths (D3D11/ANGLE, GL and Metal)
            // to align with defaults instead of forcing generic GPU flags.

            #[cfg(target_os="windows")]
            {
                // Hardware compositing
                cmd.append_switch(Some(&CefString::from("enable-direct-composition")));

                // Match Chrome default ANGLE path
                cmd.append_switch_with_value(
                    Some(&CefString::from("use-angle")),
                    Some(&CefString::from("d3d11")),
                );

                // Sandbox disable
                cmd.append_switch(Some(&CefString::from("no-sandbox")));
                cmd.append_switch(Some(&CefString::from("disable-gpu-sandbox")));
                cmd.append_switch(Some(&CefString::from("disable-setuid-sandbox")));
            }

            #[cfg(target_os="linux")]
            {
                cmd.append_switch(Some(&CefString::from("use-gl=desktop")));
                cmd.append_switch(Some(&CefString::from("enable-gpu-rasterization")));
            }

            #[cfg(target_os="macos")]
            {
                cmd.append_switch(Some(&CefString::from("enable-metal")));
            }
        }

        fn on_register_custom_schemes(
            &self,
            registrar: Option<&mut SchemeRegistrar>,
        ) {
            println!("on_register_custom_schemes called!");
            
            let registrar = registrar.unwrap();

            let flags =
                CEF_SCHEME_OPTION_STANDARD as i32 |
                CEF_SCHEME_OPTION_SECURE as i32 |
                CEF_SCHEME_OPTION_CORS_ENABLED as i32 |
                CEF_SCHEME_OPTION_FETCH_ENABLED as i32;

            let result = registrar.add_custom_scheme(
                Some(&CefString::from("app")),
                flags,
            );
            
            println!("Registered 'app://' scheme with flags {} result: {}", flags, result);
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(
                DemoBrowserProcessHandler::new(
                    self.window.clone(),
                    self.start_url.clone(),
                    RefCell::new(None),
                )
            )
        }

        fn render_process_handler(&self) -> Option<RenderProcessHandler> {
            Some(IpcRenderProcessHandler::new())
        }
    }
}
