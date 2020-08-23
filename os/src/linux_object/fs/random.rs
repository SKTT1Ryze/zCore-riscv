//! Implement INode for RandomINode

use alloc::sync::Arc;
use core::any::Any;

use rcore_fs::vfs::*;
use spin::Mutex;

/// random INode data struct
pub struct RandomINodeData {
    seed: u32,
}

/// random INode struct
#[derive(Clone)]
pub struct RandomINode {
    data: Arc<Mutex<RandomINodeData>>,
    secure: bool,
}

impl RandomINode {
    /// create a random INode
    /// - urandom -> secure = true
    /// - random -> secure = false
    pub fn new(secure: bool) -> RandomINode {
        RandomINode {
            secure,
            data: Arc::new(Mutex::new(RandomINodeData { seed: 1 })),
        }
    }
}

impl INode for RandomINode {
    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> Result<usize> {
        let mut data = self.data.lock();
        // from K&R
        for x in buf.iter_mut() {
            data.seed = data.seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            *x = (data.seed / 65536) as u8;
        }
        Ok(buf.len())
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(FsError::NotSupported)
    }

    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: true,
            write: false,
            error: false,
        })
    }

    fn metadata(&self) -> Result<Metadata> {
        Ok(Metadata {
            dev: 1,
            inode: 1,
            size: 0,
            blk_size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            type_: FileType::CharDevice,
            mode: 0o666,
            nlinks: 1,
            uid: 0,
            gid: 0,
            rdev: make_rdev(1, if self.secure { 9 } else { 8 }),
        })
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}
