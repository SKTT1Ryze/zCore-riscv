//! Implement INode for Stdin & Stdout
#![allow(unsafe_code)]

use super::ioctl::*;
use super::super::{sync::Event, sync::EventBus};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use lazy_static::lazy_static;
use rcore_fs::vfs::*;
use spin::Mutex;

lazy_static! {
    /// STDIN global reference
    pub static ref STDIN: Arc<Stdin> = Default::default();
    /// STDOUT global reference
    pub static ref STDOUT: Arc<Stdout> = Default::default();
}

/// Stdin struct, for Stdin buffer
#[derive(Default)]
pub struct Stdin {
    buf: Mutex<VecDeque<char>>,
    eventbus: Mutex<EventBus>,
}

impl Stdin {
    /// push a char in Stdin buffer
    pub fn push(&self, c: char) {
        self.buf.lock().push_back(c);
        self.eventbus.lock().set(Event::READABLE);
    }
    /// pop a char in Stdin buffer
    pub fn pop(&self) -> char {
        let mut buf_lock = self.buf.lock();
        let c = buf_lock.pop_front().unwrap();
        if buf_lock.len() == 0 {
            self.eventbus.lock().clear(Event::READABLE);
        }
        c
    }
    /// specify whether the Stdin buffer is readable
    pub fn can_read(&self) -> bool {
        self.buf.lock().len() > 0
    }
}

/// Stdout struct, empty now
#[derive(Default)]
pub struct Stdout;

impl INode for Stdin {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> Result<usize> {
        if self.can_read() {
            buf[0] = self.pop() as u8;
            Ok(1)
        } else {
            Err(FsError::Again)
        }
    }
    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }
    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: self.can_read(),
            write: false,
            error: false,
        })
    }
    fn async_poll<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<PollStatus>> + Send + Sync + 'a>> {
        #[must_use = "future does nothing unless polled/`await`-ed"]
        struct SerialFuture<'a> {
            stdin: &'a Stdin,
        };

        impl<'a> Future for SerialFuture<'a> {
            type Output = Result<PollStatus>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                if self.stdin.can_read() {
                    return Poll::Ready(self.stdin.poll());
                }
                let waker = cx.waker().clone();
                self.stdin.eventbus.lock().subscribe(Box::new({
                    move |_| {
                        waker.wake_by_ref();
                        true
                    }
                }));
                Poll::Pending
            }
        }

        Box::pin(SerialFuture { stdin: self })
    }
    /*
    fn io_control(&self, cmd: u32, data: usize) -> Result<usize> {
        match cmd as usize {
            TCGETS | TIOCGWINSZ | TIOCSPGRP => {
                // pretend to be tty
                Ok(0)
            }
            TIOCGPGRP => {
                // pretend to be have a tty process group
                // TODO: verify pointer
                unsafe { *(data as *mut u32) = 0 };
                Ok(0)
            }
            _ => Err(FsError::NotSupported),
        }
    }
    */
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

impl INode for Stdout {
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize> {
        unimplemented!()
    }
    fn write_at(&self, _offset: usize, buf: &[u8]) -> Result<usize> {
        // we do not care the utf-8 things, we just want to print it!
        let s = unsafe { core::str::from_utf8_unchecked(buf) };
        crate::kernel_hal::serial_write(s);
        Ok(buf.len())
    }
    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: false,
            write: true,
            error: false,
        })
    }
    fn io_control(&self, cmd: u32, data: usize) -> Result<usize> {
        match cmd as usize {
            TCGETS | TIOCGWINSZ | TIOCSPGRP => {
                // pretend to be tty
                Ok(0)
            }
            TIOCGPGRP => {
                // pretend to be have a tty process group
                // TODO: verify pointer
                unsafe { *(data as *mut u32) = 0 };
                Ok(0)
            }
            _ => Err(FsError::NotSupported),
        }
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}
