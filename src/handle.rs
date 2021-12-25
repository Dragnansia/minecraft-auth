use std::{mem::MaybeUninit, sync::Once};
use tokio::runtime::Handle;

pub fn main_handle() -> &'static Handle {
    static mut HANDLE: MaybeUninit<Handle> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            HANDLE.write(Handle::current());
        });
        HANDLE.assume_init_ref()
    }
}
