use futures::prelude::*;
use futures::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use futures::Future;

// This is a non operative Waker to simulate an future context
// To load asset we don't need an executor, on wasm32 web-sys will do the trick, no native the assets are loaded sync
// But we want to expose a future API to be compatible between platforms, and to do that
// the poll API forces us to pass a context.
// This code is based on this example https://github.com/jkarneges/rust-executor-example/blob/master/async.rs
static VTABLE: RawWakerVTable = RawWakerVTable::new(vt_clone, vt_dummy, vt_dummy, vt_dummy);
pub(crate) struct DummyWaker;
impl DummyWaker {
    //Noop
    pub fn into_task_waker(self) -> Waker {
        unsafe {
            let w = Box::new(self);
            let rw = RawWaker::new(Box::into_raw(w) as *mut (), &VTABLE);
            Waker::from_raw(rw)
        }
    }

    fn wake(mut self) {}

    fn wake_by_ref(&mut self) {}
}

unsafe fn vt_clone(data: *const ()) -> RawWaker {
    let w = (data as *const DummyWaker).as_ref().unwrap();
    let new_w = Box::new(w.clone());
    RawWaker::new(Box::into_raw(new_w) as *mut (), &VTABLE)
}

unsafe fn vt_dummy(data: *const ()) {}
