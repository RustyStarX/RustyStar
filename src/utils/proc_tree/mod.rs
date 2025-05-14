use std::collections::BTreeMap;

use win32_ecoqos::{
    utils::{Process, Processes},
    windows_result,
};

pub struct ProcTree {
    parent_map: BTreeMap<u32, u32>,
}

impl<I> From<I> for ProcTree
where
    I: IntoIterator<Item = Process>,
{
    fn from(iter: I) -> Self {
        let parent_map = BTreeMap::from_iter(iter.into_iter().map(
            |&Process {
                 process_id,
                 process_parent_id,
                 ..
             }| (process_id, process_parent_id),
        ));

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
        while let Some(&parent_pid) = relations.get(&pid) {
            if parent_pid == 0 || met.contains(&parent_pid) {
                return false;
            }
            if parent_pid == main_pid {
                return true;
            }

            pid = parent_pid;
            met.insert(pid);
        }

        false
    }
}
