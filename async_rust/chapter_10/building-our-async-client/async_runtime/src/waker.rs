use std::task::{RawWaker, RawWakerVTable};


static VTABLE: RawWakerVTable = RawWakerVTable::new(
    my_clone,
    my_wake,
    my_wake_by_ref,
    my_drop,
);

unsafe fn my_clone(raw_waker: *const ()) -> RawWaker {
    RawWaker::new(raw_waker, &VTABLE)
}

unsafe fn my_wake(raw_waker: *const ()) {
    drop(Box::from_raw(raw_waker as *mut u32));
}

unsafe fn my_wake_by_ref(_raw_waker: *const ()) {
}

unsafe fn my_drop(raw_waker: *const ()) {
    drop(Box::from_raw(raw_waker as *mut u32));
}

pub fn create_raw_waker() -> RawWaker {
    let data = Box::into_raw(Box::new(42u32));
    RawWaker::new(data as *const (), &VTABLE)
}