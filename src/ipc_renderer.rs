//! Renderer process IPC implementation.
//! Transport uses ProcessMessage "ipc" and ListValue typed args.

use cef::*;
use cef::rc::*;
use std::sync::Mutex;
use std::collections::HashMap;

/// Tracks pending promises awaiting responses from the browser process
struct PromiseRegistry {
    next_id: u32,
    pending: HashMap<u32, (V8Context, V8Value)>,
}

impl PromiseRegistry {
    fn new() -> Self {
        Self {
            next_id: 1,
            pending: HashMap::new(),
        }
    }

    fn register(&mut self, context: V8Context, promise: V8Value) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.pending.insert(id, (context, promise));
        id
    }

    fn resolve(&mut self, id: u32, success: bool, result: &str) {
        if let Some((context, promise)) = self.pending.remove(&id) {
            if context.enter() == 0 {
                eprintln!("Failed to enter V8 context for promise resolution");
                return;
            }

            let s = CefString::from(result);

            if success {
                // resolve promise with string value
                let mut v = v8_value_create_string(Some(&s)).unwrap();
                promise.resolve_promise(Some(&mut v));
            } else {
                // reject with exception string
                promise.reject_promise(Some(&s));
            }

            context.exit();
        } else {
            eprintln!("No pending promise found for ID {}", id);
        }
    }
}

static PROMISE_REGISTRY: Mutex<Option<PromiseRegistry>> = Mutex::new(None);

fn ensure_registry() {
    let mut g = PROMISE_REGISTRY.lock().unwrap();
    if g.is_none() {
        *g = Some(PromiseRegistry::new());
    }
}

fn register_promise(context: V8Context, promise: V8Value) -> u32 {
    let mut guard = PROMISE_REGISTRY.lock().unwrap();
    guard.as_mut().unwrap().register(context, promise)
}

fn resolve_promise(id: u32, success: bool, result: &str) {
    let mut guard = PROMISE_REGISTRY.lock().unwrap();
    guard.as_mut().unwrap().resolve(id, success, result)
}

fn list_int(args: &ListValue, idx: usize) -> i32 {
    args.int(idx)
}

fn list_string_to_rust(args: &ListValue, idx: usize) -> String {
    let userfree = args.string(idx);
    let cef: CefString = (&userfree).into();
    cef.to_string()
}

wrap_render_process_handler! {
    pub struct IpcRenderProcessHandler;

    impl RenderProcessHandler {
        fn on_context_created(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            ensure_registry();

            let context = context.unwrap();
            let global = context.global().unwrap();

            let mut core = v8_value_create_object(None, None).unwrap();

            let mut handler = IpcInvokeHandler::new();
            let mut invoke = v8_value_create_function(
                Some(&CefString::from("invoke")),
                Some(&mut handler),
            ).unwrap();

            core.set_value_bykey(
                Some(&CefString::from("invoke")),
                Some(&mut invoke),
                V8Propertyattribute::default(),
            );

            global.set_value_bykey(
                Some(&CefString::from("core")),
                Some(&mut core),
                V8Propertyattribute::default(),
            );

            println!("[Renderer] Injected window.core.invoke");
        }

        fn on_process_message_received(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> i32 {
            if source_process != ProcessId::BROWSER {
                return 0;
            }

            let msg = match message {
                Some(m) => m,
                None => return 0,
            };

            let name: CefString = (&msg.name()).into();
            if name.to_string() != "ipc" {
                return 0;
            }

            let args = match msg.argument_list() {
                Some(a) => a,
                None => return 0,
            };

            let msg_type = list_int(&args, 0);
            let id = list_int(&args, 1) as u32;
            let payload = list_string_to_rust(&args, 2);

            match msg_type {
                1 => resolve_promise(id, true, &payload),
                2 => resolve_promise(id, false, &payload),
                _ => {
                    eprintln!("[Renderer] Unknown ipc message type: {}", msg_type);
                }
            }

            1
        }
    }
}

wrap_v8_handler! {
    pub struct IpcInvokeHandler;

    impl V8Handler {
        fn execute(
            &self,
            _name: Option<&CefString>,
            _object: Option<&mut V8Value>,
            arguments: Option<&[Option<V8Value>]>,
            retval: Option<&mut Option<V8Value>>,
            exception: Option<&mut CefString>,
        ) -> i32 {
            // args must be present
            let args = match arguments {
                Some(a) => a,
                None => {
                    if let Some(exc) = exception {
                        *exc = CefString::from("No arguments");
                    }
                    return 0;
                }
            };

            if args.len() < 1 {
                if let Some(exc) = exception {
                    *exc = CefString::from("invoke requires at least 1 argument");
                }
                return 0;
            }

            // first arg: command string
            let cmd = match &args[0] {
                Some(v) if v.is_string() != 0 => {
                    let s = v.string_value();
                    let cef: CefString = (&s).into();
                    cef.to_string()
                }
                _ => {
                    if let Some(exc) = exception {
                        *exc = CefString::from("command must be a string");
                    }
                    return 0;
                }
            };

            // optional payload (string)
            let payload = if args.len() > 1 {
                if let Some(Some(v)) = args.get(1) {
                    if v.is_string() != 0 {
                        let s = v.string_value();
                        let cef: CefString = (&s).into();
                        cef.to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let context = v8_context_get_current_context().unwrap();
            let promise = v8_value_create_promise().unwrap();

            let id = register_promise(context.clone(), promise.clone());

            println!("[Renderer] JS invoke: {} (id={})", cmd, id);

            if let Some(frame) = context.frame() {
                let mut msg = process_message_create(Some(&CefString::from("ipc"))).unwrap();
                let args = msg.argument_list().unwrap();

                args.set_int(0, 0); // invoke
                args.set_int(1, id as i32);
                args.set_string(2, Some(&CefString::from(cmd.as_str())));
                args.set_string(3, Some(&CefString::from(payload.as_str())));

                frame.send_process_message(ProcessId::BROWSER, Some(&mut msg));
            }

            if let Some(ret) = retval {
                *ret = Some(promise);
            }

            1
        }
    }
}
