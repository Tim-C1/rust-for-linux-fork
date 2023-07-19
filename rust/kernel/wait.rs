use core::fmt;

use crate::bindings::wait_queue_head_t;
use crate::bindings::lock_class_key;
use crate::Result;
use crate::pr_info;

/// waitqueue
pub struct WaitQueue {
    pub foo: i32,
    pub bindings: bindings::wait_queue_head_t,
}

unsafe impl Sync for WaitQueue {}
unsafe impl Send for WaitQueue {}

impl WaitQueue {
    /// try new
    pub fn try_new(name: fmt::Arguments<'_>) -> Result<WaitQueue> {
        unsafe {
            let mut __key = lock_class_key::default();
            let mut wq = wait_queue_head_t::default();
            bindings::__init_waitqueue_head(
                &mut wq as *mut _,
                &name as *const _ as *const core::ffi::c_char,
                &mut __key as *mut _
            );
            return Ok(WaitQueue{
                foo: 1,
                bindings: wq,
            });
        };
    }

    pub fn try_wait(&mut self) {
        // pr_info!("I am here\n");
        pr_info!("before schedule, foo {}", self.foo);
        unsafe {
            let mut wq_entry = bindings::wait_queue_entry::default();
            bindings::init_wait(&mut wq_entry as *mut _);
            bindings::prepare_to_wait(&mut self.bindings as *mut _, &mut wq_entry as *mut _, bindings::TASK_INTERRUPTIBLE.try_into().unwrap());
            bindings::schedule();
            bindings::finish_wait(&mut self.bindings as *mut _, &mut wq_entry as *mut _);
        }
        pr_info!("after schedule");
        // pr_info!("finish wait\n");
    }

    pub fn try_wake(&mut self) -> i32 {
        unsafe {
            let rst = bindings::__wake_up(&mut self.bindings as *mut _, bindings::TASK_NORMAL, 1, core::ptr::null_mut() as *mut _ as *mut core::ffi::c_void);
            return rst;
        }
    }
}
