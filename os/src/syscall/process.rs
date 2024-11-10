//! Process management syscalls
use crate::{

    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, 
    mm::{translated_byte_buffer, MapPermission, VirtAddr},

    task::*,

    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let src = &time_val as *const TimeVal;
    
    let mut src = src as usize;
    
    let dst_vec = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    for dst in dst_vec {
        unsafe {
            core::ptr::copy_nonoverlapping(src as *mut u8, dst.as_mut_ptr(), dst.len());
            src += dst.len();
        }
    }
    0
}

/// Records the number of system calls for the current task
pub fn sys_record_syscall(syscall_id: usize) -> isize {
    trace!("kernel: sys_record_syscall, syscall_id={}", syscall_id);
    record_current_task_syscall_times(syscall_id);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let task_info = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_times(),
        time: get_time_ms() - get_current_task_first_start_time(),
    };
   
    let src = &task_info as *const TaskInfo;
   
    let mut src = src as usize;
    
    let dst_vec = translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());
    for dst in dst_vec {
        unsafe {
            core::ptr::copy_nonoverlapping(src as *mut u8, dst.as_mut_ptr(), dst.len());
            src += dst.len();
        }
    }
    0
}


/// Records the number of system calls for the current task
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _start % PAGE_SIZE != 0 ||
        _port & !0x7 != 0 ||
        _port & 0x7 == 0 {
        return -1;
    }
    // Using from_bits_truncate because our flags contain unknown bits, so from_bits cannot be used
    let permission = MapPermission::from_bits_truncate((_port << 1) as u8) | MapPermission::U;
    // Creates a new memory region
    // Rounds down and up
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    // Calls a function from the task module
    //* [start, start + len) contains already mapped pages */
    create_new_map_area(start_vpn, end_vpn, permission)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    // Creates a new memory region
    // Rounds down and up
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    //* [start, start + len) contains unmapped pages */
    remove_map_area(start_vpn, end_vpn)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
