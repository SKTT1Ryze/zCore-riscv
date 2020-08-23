//! Linux semaphore ipc
use super::super::error::LxError;
use super::super::sync::Semaphore;
use super::super::time::*;
use alloc::{collections::BTreeMap, sync::Arc, sync::Weak, vec::Vec};
use bitflags::*;
use core::ops::Index;
use lazy_static::*;
use spin::Mutex;
use spin::RwLock;

bitflags! {
    struct SemGetFlag: usize {
        const CREAT = 1 << 9;
        const EXCLUSIVE = 1 << 10;
        const NO_WAIT = 1 << 11;
    }
}

/// structure specifies the access permissions on the semaphore set
///
/// struct ipc_perm
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpcPerm {
    /// Key supplied to semget(2)
    pub key: u32,
    /// Effective UID of owner
    pub uid: u32,
    /// Effective GID of owner
    pub gid: u32,
    /// Effective UID of creator
    pub cuid: u32,
    /// Effective GID of creator
    pub cgid: u32,
    /// Permissions
    pub mode: u32,
    /// Sequence number
    pub __seq: u32,
    /// pad1
    pub __pad1: usize,
    /// pad2
    pub __pad2: usize,
}

/// semid data structure
///
/// struct semid_ds
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SemidDs {
    /// Ownership and permissions
    pub perm: IpcPerm,
    /// Last semop time
    pub otime: usize,
    __pad1: usize,
    /// Last change time
    pub ctime: usize,
    __pad2: usize,
    /// number of semaphores in set
    pub nsems: usize,
}

/// A System V semaphore set
pub struct SemArray {
    /// semid data structure
    pub semid_ds: Mutex<SemidDs>,
    sems: Vec<Semaphore>,
}

impl Index<usize> for SemArray {
    type Output = Semaphore;
    fn index(&self, idx: usize) -> &Semaphore {
        &self.sems[idx]
    }
}

lazy_static! {
    static ref KEY2SEM: RwLock<BTreeMap<u32, Weak<SemArray>>> = RwLock::new(BTreeMap::new());
}

impl SemArray {
    /// remove semaphores
    pub fn remove(&self) {
        let mut key2sem = KEY2SEM.write();
        let key = self.semid_ds.lock().perm.key;
        key2sem.remove(&key);
        for sem in self.sems.iter() {
            sem.remove();
        }
    }

    /// set last semop time
    pub fn otime(&self) {
        self.semid_ds.lock().otime = TimeSpec::now().sec;
    }

    /// set last change time
    pub fn ctime(&self) {
        self.semid_ds.lock().ctime = TimeSpec::now().sec;
    }

    /// for IPC_SET
    /// see man semctl(2)
    pub fn set(&self, new: &SemidDs) {
        let mut lock = self.semid_ds.lock();
        lock.perm.uid = new.perm.uid;
        lock.perm.gid = new.perm.gid;
        lock.perm.mode = new.perm.mode & 0x1ff;
    }

    /// Get the semaphore array with `key`.
    /// If not exist, create a new one with `nsems` elements.
    pub fn get_or_create(mut key: u32, nsems: usize, flags: usize) -> Result<Arc<Self>, LxError> {
        let mut key2sem = KEY2SEM.write();
        let flag = SemGetFlag::from_bits_truncate(flags);

        if key == 0 {
            // IPC_PRIVATE
            // find an empty key slot
            key = (1u32..).find(|i| key2sem.get(i).is_none()).unwrap();
        } else {
            // check existence
            if let Some(weak_array) = key2sem.get(&key) {
                if let Some(array) = weak_array.upgrade() {
                    if flag.contains(SemGetFlag::CREAT) && flag.contains(SemGetFlag::EXCLUSIVE) {
                        // exclusive
                        return Err(LxError::EEXIST);
                    }
                    return Ok(array);
                }
            }
        }

        // not found, create one
        let mut semaphores = Vec::new();
        for _ in 0..nsems {
            semaphores.push(Semaphore::new(0));
        }

        // insert to global map
        let array = Arc::new(SemArray {
            semid_ds: Mutex::new(SemidDs {
                perm: IpcPerm {
                    key,
                    uid: 0,
                    gid: 0,
                    cuid: 0,
                    cgid: 0,
                    // least significant 9 bits
                    mode: (flags as u32) & 0x1ff,
                    __seq: 0,
                    __pad1: 0,
                    __pad2: 0,
                },
                otime: 0,
                ctime: TimeSpec::now().sec,
                nsems,
                __pad1: 0,
                __pad2: 0,
            }),
            sems: semaphores,
        });
        key2sem.insert(key, Arc::downgrade(&array));
        Ok(array)
    }
}
