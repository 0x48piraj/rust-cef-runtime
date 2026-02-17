//! Browser client implementation.

use cef::*;
use cef::rc::*;

wrap_client! {
    pub struct DemoClient;

    impl Client {
        fn on_process_message_received(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> i32 {
            // Only handle messages from renderer
            if source_process != ProcessId::RENDERER {
                return 0;
            }

            let browser = browser.unwrap();
            let frame = frame.unwrap();
            let msg = message.unwrap();

            // Delegate to IPC dispatcher
            if crate::ipc_browser::handle_ipc_message(browser, frame, msg) {
                return 1;
            }

            0
        }
    }
}
