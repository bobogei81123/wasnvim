use std::{cell::RefCell, rc::Rc};

use exports::nvim::api::client_callback_impl::{Callback, ClientCallbackImpl};
use nvim::api::nvim_api::{nvim_echo, nvim_err_writeln};

wit_bindgen::generate!({
    exports: {
        world: MyPluginImp,
        "nvim:api/client-callback-impl": ClientCallbackImp,
    }
});

pub struct MyPluginImp;

impl MyPlugin for MyPluginImp {
    fn run(args: Vec<Object>) -> Object {
        let mut x = 0;
        let handle = CallbackImpl::new(move || {
            x += 1;
            nvim_err_writeln(&format!("called {} time", x));
        })
        .into_handle();
        nvim_err_writeln(&format!("create callback {}", handle as i32));
        test_register_callback(handle);

        ClientCallbackImp::call_callback(handle as i32, vec![]);
        ClientCallbackImp::call_callback(handle as i32, vec![]);
        ClientCallbackImp::call_callback(handle as i32, vec![]);

        Object::Nil
    }
}

pub struct ClientCallbackImp;

impl ClientCallbackImpl for ClientCallbackImp {
    fn drop_callback(callback_ref: Callback) {
        CallbackImpl::from_handle(callback_ref as u32);
    }
    fn call_callback(callback_ref: Callback, _args: Vec<Object>) {
        nvim_err_writeln(&format!("call callback {}", callback_ref));
        let cb = CallbackImpl::from_handle(callback_ref as u32);
        cb.call();
        std::mem::forget(cb);
    }
}

pub struct CallbackImpl {
    inner: Rc<RefCell<Box<dyn FnMut()>>>,
}

impl CallbackImpl {
    fn new<F: FnMut() + 'static>(f: F) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Box::new(f))),
        }
    }

    fn into_handle(self) -> u32 {
        Rc::into_raw(self.inner) as usize as u32
    }

    fn from_handle(id: u32) -> Self {
        Self {
            inner: unsafe { Rc::from_raw(id as usize as *const RefCell<Box<dyn FnMut()>>) },
        }
    }

    fn call(&self) {
        self.inner.borrow_mut()()
    }
}
