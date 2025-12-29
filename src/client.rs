//! Browser client implementation.

use cef::*;
use cef::rc::*;

wrap_client! {
    pub struct DemoClient;

    impl Client {
        // Scheme handler factory handles all app:// requests
    }
}
