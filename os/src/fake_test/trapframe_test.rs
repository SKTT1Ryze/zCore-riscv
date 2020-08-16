use trapframe::{GeneralRegs,UserContext, TrapFrame};
use riscv::register::{scause, stval};
use riscv::register::scause::{Exception as E, Trap};
use crate::{print, println};

pub fn trapframe_test() {
    unsafe {
        trapframe::init();
    }
    println!("test trapframe");

    let mut regs = UserContext {
        general: GeneralRegs {
            zero: 0,
            ra: 1,
            sp: 0x8080_0000,
            gp: 3,
            tp: 4,
            t0: 5,
            t1: 6,
            t2: 7,
            s0: 8,
            s1: 9,
            a0: 10,
            a1: 11,
            a2: 12,
            a3: 13,
            a4: 14,
            a5: 15,
            a6: 16,
            a7: 17,
            s2: 18,
            s3: 19,
            s4: 20,
            s5: 21,
            s6: 22,
            s7: 23,
            s8: 24,
            s9: 25,
            s10: 26,
            s11: 27,
            t3: 28,
            t4: 29,
            t5: 30,
            t6: 31,
        },
        sstatus: 0xdead_beaf,
        sepc: user_entry as usize,
    };
    println!("Go to user: {:#x?}", regs);
    regs.run();
    let scause = scause::read();
    let stval = stval::read();
    println!(
        "Back from user: {:?}, stval={:#x}\n{:#x?}",
        scause.cause(),
        stval,
        regs
    );
    unsafe {
        asm!("ebreak");
    }

    println!("Exit...");

}
#[no_mangle]
extern "C" fn trap_handler(tf: &mut TrapFrame) {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => {
            println!("TRAP: Breakpoint");
            tf.sepc += 2;
        }
        _ => panic!(
            "TRAP: scause={:?}, stval={:#x}, tf={:#x?}",
            scause.cause(),
            stval,
            tf
        ),
    }
}

unsafe extern "C" fn user_entry() {
    println!("user_entry()");
}