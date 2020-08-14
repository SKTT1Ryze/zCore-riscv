use crate::kernel_hal_bare::Frame;
use alloc::vec::Vec;
pub fn frame_test() {
    let mut frame_vec = Vec::new();
    for _ in 0..10 {
        let new_frame = Frame::alloc();
        match new_frame {
            None => {
                panic!("alloc new frame error");
            },
            Some(frame) => {
                frame_vec.push(frame);
            }
        }
    }
    assert_eq!(frame_vec.len(), 10);
    for i in 0..10 {
        frame_vec[i].dealloc();
    }
    println!("frame_test pass");
}

#[no_mangle]
pub extern "C" fn hal_frame_alloc() -> Option<usize> {
    println!("running in hal_frame_alloc()");
    Some(0)
}

#[no_mangle]
pub extern "C" fn hal_frame_dealloc() -> Option<usize> {
    println!("running in hal_frame_dealloc()");
    Some(1)
}