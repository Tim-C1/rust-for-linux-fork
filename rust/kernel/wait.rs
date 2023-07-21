use core::fmt;

use crate::bindings::lock_class_key;
use crate::bindings::wait_queue_head;
use crate::pr_info;
use crate::Result;

/// waitqueue
pub struct WaitQueue {
    pub bindings: bindings::wait_queue_head,
}

unsafe impl Sync for WaitQueue {}
unsafe impl Send for WaitQueue {}

impl WaitQueue {
    /// try new
    pub fn try_new(name: fmt::Arguments<'_>) -> Result<WaitQueue> {
        unsafe {
            let mut __key = lock_class_key::default();
            let mut wq = wait_queue_head::default();
            bindings::__init_waitqueue_head(
                &mut wq as *mut _,
                &name as *const _ as *const core::ffi::c_char,
                &mut __key as *mut _,
            );
            return Ok(WaitQueue { bindings: wq });
        };
    }

    pub fn wait_event_interruptible(&mut self, cond: bool) -> i64 {
        let mut __ret: i64 = 0;
        if !cond {
            __ret = self.__wait_event_interruptible(cond);
        }
        return __ret;
    }

    fn __wait_event_interruptible(&mut self, cond: bool) -> i64 {
        self.___wait_event(cond, bindings::TASK_INTERRUPTIBLE.try_into().unwrap(), 0, 0)
    }

    fn ___wait_event(&mut self, cond: bool, state: i32, _exclusive: i32, ret: i64) -> i64 {
            let mut __ret: i64 = ret;
            let mut __wq_entry = bindings::wait_queue_entry::default();
            unsafe { bindings::init_wait_entry(&mut __wq_entry as *mut _, 0) };
            loop {
                let __int: i64 = unsafe { bindings::prepare_to_wait_event(
                    &mut self.bindings as *mut _,
                    &mut __wq_entry as *mut _,
                    state,
                ) };

                if cond {
                    break;
                }
                //(state & (TASK_INTERRUPTIBLE | TASK_WAKEKILL))) && __int
                if ((state as u32 & (bindings::TASK_INTERRUPTIBLE | bindings::TASK_WAKEKILL))) > 0 && __int > 0 {
                    __ret = __int;
                    return __ret;
                }

                pr_info!("before schedule\n");
                unsafe { bindings::schedule() };
            }
            unsafe { bindings::finish_wait(&mut self.bindings as *mut _, &mut __wq_entry as *mut _) };
            return 0;
    }

    pub fn try_wake(&mut self) -> i32 {
        unsafe {
            bindings::__wake_up(
                &mut self.bindings as *mut _,
                bindings::TASK_NORMAL,
                1,
                core::ptr::null_mut() as *mut _ as *mut core::ffi::c_void,
            );
        }
        pr_info!("finish wake up");
        return 0;
    }
}
