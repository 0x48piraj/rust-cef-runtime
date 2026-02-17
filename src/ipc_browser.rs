//! Browser process IPC dispatcher.
//! Commands may be registered before CEF starts.
//! They are buffered and installed once the browser process initializes.

use cef::*;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

pub type IpcResult = Result<String, String>;
pub type IpcHandler = Box<dyn Fn(&str) -> IpcResult + Send + Sync>;

pub struct IpcDispatcher {
    handlers: HashMap<String, IpcHandler>,
}

impl IpcDispatcher {
    fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(&mut self, command: impl Into<String>, handler: IpcHandler) {
        self.handlers.insert(command.into(), handler);
    }

    fn dispatch(&self, command: &str, payload: &str) -> IpcResult {
        if let Some(handler) = self.handlers.get(command) {
            handler(payload)
        } else {
            Err(format!("Unknown command: {}", command))
        }
    }
}

//
// Global state
//

/// Live dispatcher (exists only after CEF browser process starts)
static DISPATCHER: OnceLock<Arc<Mutex<IpcDispatcher>>> = OnceLock::new();

/// Commands registered before runtime boot
static PENDING_COMMANDS: OnceLock<Mutex<Vec<(String, IpcHandler)>>> = OnceLock::new();

fn pending_commands() -> &'static Mutex<Vec<(String, IpcHandler)>> {
    PENDING_COMMANDS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Called by runtime when browser process initializes.
/// Drains any commands registered before init.
pub fn init_dispatcher() -> Arc<Mutex<IpcDispatcher>> {
    let dispatcher = DISPATCHER
        .get_or_init(|| Arc::new(Mutex::new(IpcDispatcher::new())))
        .clone();

    // Drain pending registrations (drop guards before returning)
    {
        let mut pending = pending_commands().lock().unwrap();
        let mut disp = dispatcher.lock().unwrap();

        for (cmd, handler) in pending.drain(..) {
            disp.register(cmd, handler);
        }
    }

    dispatcher
}

/// Get dispatcher AFTER init
pub fn get_dispatcher() -> Arc<Mutex<IpcDispatcher>> {
    DISPATCHER
        .get()
        .expect("IPC dispatcher not initialized")
        .clone()
}

/// Safe to call BEFORE runtime starts
pub fn register_command<F>(command: impl Into<String>, handler: F)
where
    F: Fn(&str) -> IpcResult + Send + Sync + 'static,
{
    let handler: IpcHandler = Box::new(handler);

    if let Some(dispatcher) = DISPATCHER.get() {
        dispatcher.lock().unwrap().register(command, handler);
    } else {
        pending_commands().lock().unwrap().push((command.into(), handler));
    }
}

//
// Message handling (structured transport using ProcessMessage + ListValue)
//

fn list_get_int(args: &ListValue, idx: usize) -> i32 {
    // binding exposes .int(index)
    args.int(idx)
}

fn list_get_string(args: &ListValue, idx: usize) -> String {
    // binding exposes .string(index) -> CefStringUserfree
    let userfree = args.string(idx);
    // Convert to CefString (borrow conversion) then to Rust String
    let cef: CefString = (&userfree).into();
    cef.to_string()
}

pub fn handle_ipc_message(
    _browser: &mut Browser,
    frame: &mut Frame,
    message: &mut ProcessMessage,
) -> bool {
    let name: CefString = (&message.name()).into();
    if name.to_string() != "ipc" {
        return false;
    }

    let args = match message.argument_list() {
        Some(a) => a,
        None => return false,
    };

    // Message type: 0 = invoke, 1 = resolve (browser shouldn't receive), 2 = reject
    let msg_type = list_get_int(&args, 0);
    if msg_type != 0 {
        // Browser only handles invokes
        return false;
    }

    let id = list_get_int(&args, 1) as u32;
    let command = list_get_string(&args, 2);
    let payload = list_get_string(&args, 3);

    println!("[Browser] IPC invoke: '{}' (id={})", command, id);

    let dispatcher = get_dispatcher();
    let result = dispatcher.lock().unwrap().dispatch(&command, &payload);

    send_response(frame, id, result);

    true
}

fn send_response(frame: &mut Frame, id: u32, result: IpcResult) {
    let mut msg = process_message_create(Some(&CefString::from("ipc"))).unwrap();
    let args = msg.argument_list().unwrap();

    match result {
        Ok(payload) => {
            args.set_int(0, 1); // resolve
            args.set_int(1, id as i32);
            args.set_string(2, Some(&CefString::from(payload.as_str())));
        }
        Err(err) => {
            args.set_int(0, 2); // reject
            args.set_int(1, id as i32);
            args.set_string(2, Some(&CefString::from(err.as_str())));
        }
    }

    frame.send_process_message(ProcessId::RENDERER, Some(&mut msg));
}
