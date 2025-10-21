use rustc_hash::FxHashSet;
use std::collections::BTreeMap;

use win32_ecoqos::{
    utils::{Process, Processes},
    windows_result,
};

pub trait ProcessInfo {
    fn pid(&self) -> u32;
    fn parent_pid(&self) -> u32;
}

pub struct ProcTree {
    parent_map: BTreeMap<u32, u32>,
}

impl<I, P> From<I> for ProcTree
where
    I: IntoIterator<Item = P>,
    P: ProcessInfo,
{
    fn from(iter: I) -> Self {
        let parent_map = BTreeMap::from_iter(iter.into_iter().map(|p| (p.pid(), p.parent_pid())));
        Self { parent_map }
    }
}

impl ProcTree {
    pub fn new() -> windows_result::Result<Self> {
        Ok(Self::from(Processes::try_new()?))
    }

    pub fn is_in_tree(&self, root: u32, mut pid: u32) -> bool {
        // first case: it self is root process
        if pid == root {
            return true;
        }

        let mut met = FxHashSet::default();
        while let Some(&parent_pid) = self.parent_map.get(&pid) {
            if parent_pid == 0 || met.contains(&parent_pid) {
                return false;
            }
            if parent_pid == root {
                return true;
            }

            pid = parent_pid;
            met.insert(pid);
        }

        false
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> u32 {
        self.process_id
    }

    fn parent_pid(&self) -> u32 {
        self.process_parent_id
    }
}

impl<'a, P> ProcessInfo for &'a P
where
    P: ProcessInfo,
{
    fn pid(&self) -> u32 {
        P::pid(&self)
    }

    fn parent_pid(&self) -> u32 {
        P::parent_pid(&self)
    }
}
