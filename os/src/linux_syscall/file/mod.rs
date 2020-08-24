//! Syscalls for files
#![deny(missing_docs)]
use super::*;
use bitflags::bitflags;
use crate::linux_object::fs::vfs::{FileType, FsError};
use crate::linux_object::fs::*;

mod dir;
mod fd;
#[allow(clippy::module_inception)]
mod file;
mod poll;
mod stat;

use self::dir::AtFlags;
