use web_sys::{
    js_sys,
    wasm_bindgen::{self, JsValue},
    Worker, WorkerOptions, WorkerType,
};

// https://github.com/trunk-rs/trunk/blob/main/examples/webworker-module/src/bin/app.rs
fn worker_new(url: &str) -> Worker {
    let options = WorkerOptions::new();
    options.set_type(WorkerType::Module);
    Worker::new_with_options(&url, &options).expect("failed to spawn worker")
}

pub fn prep_worker() {
    let worker = worker_new("./shaderworker_loader.js");
    let worker_clone = worker.clone();
}

// https://www.tweag.io/blog/2022-11-24-wasm-threads-and-messages/
pub fn spawn(f: impl FnOnce() + Send + 'static) -> Result<web_sys::Worker, JsValue> {
    let worker = web_sys::Worker::new("./shaderworker.js")?;
    // Double-boxing because `dyn FnOnce` is unsized and so `Box<dyn FnOnce()>` is a fat pointer.
    // But `Box<Box<dyn FnOnce()>>` is just a plain pointer, and since wasm has 32-bit pointers,
    // we can cast it to a `u32` and back.
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));
    let msg = js_sys::Array::new();
    // Send the worker a reference to our memory chunk, so it can initialize a wasm module
    // using the same memory.
    msg.push(&wasm_bindgen::memory());
    // Also send the worker the address of the closure we want to execute.
    msg.push(&JsValue::from(ptr as u32));
    worker.post_message(&msg);
    Ok(worker)
}

// https://github.com/trunk-rs/trunk/tree/main/examples/webworker-module

use js_sys::Array;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

pub fn worker_main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"worker starting".into());

    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()));
    let scope_clone = scope.clone();

    let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
        web_sys::console::log_1(&"got message".into());

        let data = Array::from(&msg.data());
        let a = data
            .get(0)
            .as_f64()
            .expect("first array value to be a number") as u32;
        let b = data
            .get(1)
            .as_f64()
            .expect("second array value to be a number") as u32;

        data.push(&(a * b).into());
        scope_clone
            .post_message(&data.into())
            .expect("posting result message succeeds");
    }) as Box<dyn Fn(MessageEvent)>);
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // The worker must send a message to indicate that it's ready to receive messages.
    scope
        .post_message(&Array::new().into())
        .expect("posting ready message succeeds");
}
