use std::ffi::c_void;

use windows_sys::Win32::{Foundation::{EXCEPTION_BREAKPOINT, EXCEPTION_SINGLE_STEP, STATUS_SUCCESS}, System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, CONTEXT_DEBUG_REGISTERS_AMD64, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS
}};

fn main() {
    println!("Starting program..");
    let _h = unsafe { AddVectoredExceptionHandler(1, Some(veh)) };
    unsafe { core::arch::asm!("int3") };
    change_execution();
    println!("Finished!")
}

#[inline(never)]
fn change_execution() {
    println!("If this worked I should not print!!!! :(");
}

unsafe extern "system" fn veh(p_ep: *mut EXCEPTION_POINTERS) -> i32 {
    let exception_record = unsafe { *(*p_ep).ExceptionRecord  };
    let ctx = unsafe { &mut *(*p_ep).ContextRecord };

    if exception_record.ExceptionCode == EXCEPTION_BREAKPOINT {
        println!("Received initial break to set hardware breakpoint on a function");
        // Set the address we wish to monitor for a hardware breakpoint
        ctx.Dr0 = change_execution as *const c_void as u64;
        // Set the bit which says Dr0 is enabled locally
        ctx.Dr7 |= 1;
        // Increase the instruction pointer by 1, so we effectively move to the next instruction after int3
        ctx.Rip += 1;
        // Set flags 
        ctx.ContextFlags |= CONTEXT_DEBUG_REGISTERS_AMD64;
        // clear dr6
        ctx.Dr6 = 0;

        return EXCEPTION_CONTINUE_EXECUTION;
    } else if exception_record.ExceptionCode == EXCEPTION_SINGLE_STEP {

        // Gate the exception to make sure it was our entry which triggered
        // to prevent false positives (will probably lead to UB in the process)
        if (ctx.Dr6 & 0x1) == 0 {
            return EXCEPTION_CONTINUE_SEARCH;
        }

        println!("Now in the 2nd VEH when change_execution was accessed");

        // fake a return value as if we intercepted a syscall
        ctx.Rax = STATUS_SUCCESS as u64;

        // get return addr from the stack
        let rsp = ctx.Rsp as *const u64;
        let return_address = unsafe { *rsp };
        // set it
        ctx.Rip = return_address;

        // simulate popping the ret from the stack
        ctx.Rsp += 8;

        // clear dr6
        ctx.Dr6 = 0;
        return EXCEPTION_CONTINUE_EXECUTION;
    }

    EXCEPTION_CONTINUE_SEARCH
}