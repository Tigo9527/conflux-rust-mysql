use super::super::errors::*;
use crate::{
    impls::{
        delta_mpt::node_ref_map::DeltaMptId, single_mpt_state::SingleMptState,
        state_manager::DeltaDbManager,
    },
    node_memory_manager::{
        DeltaMptsCacheAlgorithm, DeltaMptsNodeMemoryManager,
    },
    storage_db::DeltaDbManagerTrait,
    ArcDeltaDbWrapper, CowNodeRef, DeltaMpt, OpenableOnDemandOpenDeltaDbTrait,
};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use parking_lot::Mutex;
use primitives::EpochId;
use std::{fs, path::PathBuf, sync::Arc};

const DB_NAME: &str = "single_mpt";

pub struct SingleMptStorageManager {
    db_manager: Arc<SingleMptDbManager>,
    node_memory_manager: Arc<DeltaMptsNodeMemoryManager>,
    mpt: Arc<DeltaMpt>,
}

impl SingleMptStorageManager {
    pub fn new_arc(db_path: PathBuf) -> Arc<Self> {
        if !db_path.exists() {
            fs::create_dir_all(&db_path).expect("db path create error");
        }
        let db_manager = Arc::new(SingleMptDbManager {
            db_manager: DeltaDbManager::new(db_path)
                .expect("DeltaDb initialize error"),
            opened_mpt: Mutex::new(None),
        });
        let node_memory_manager = Arc::new(DeltaMptsNodeMemoryManager::new(
            1_000_000,
            10_000_000,
            1_000_000,
            1_000_000,
            DeltaMptsCacheAlgorithm::new(10_000_000),
        ));
        let mpt = Arc::new(
            DeltaMpt::new_single_mpt(
                db_manager.clone(),
                node_memory_manager.clone(),
            )
            .expect("MPT initialization error"),
        );
        Arc::new(Self {
            db_manager,
            node_memory_manager,
            mpt,
        })
    }

    pub fn get_state_by_epoch(
        &self, epoch: EpochId,
    ) -> Result<Option<SingleMptState>> {
        let root = self.mpt.get_root_node_ref_by_epoch(&epoch)?;
        match root {
            Some(Some(root)) => {
                Ok(Some(SingleMptState::new(self.mpt.clone(), root)))
            }
            _ => Ok(None),
        }
    }

    pub fn get_state_for_genesis(&self) -> Result<SingleMptState> {
        Ok(SingleMptState::new_empty(self.mpt.clone()))
    }
}

struct SingleMptDbManager {
    db_manager: DeltaDbManager,
    opened_mpt: Mutex<Option<ArcDeltaDbWrapper>>,
}

impl OpenableOnDemandOpenDeltaDbTrait for SingleMptDbManager {
    fn open(&self, mpt_id: DeltaMptId) -> Result<ArcDeltaDbWrapper> {
        if mpt_id == 0 {
            let mut maybe_mpt = self.opened_mpt.lock();
            if maybe_mpt.is_some() {
                return Ok(maybe_mpt.as_ref().unwrap().clone());
            }
            let db = match self.db_manager.get_delta_db(DB_NAME)? {
                Some(db) => db,
                None => self.db_manager.new_empty_delta_db(DB_NAME)?,
            };
            let mpt = ArcDeltaDbWrapper {
                inner: Some(Arc::new(db)),
                lru: None,
                mpt_id,
            };
            *maybe_mpt = Some(mpt.clone());
            Ok(mpt)
        } else {
            Err(ErrorKind::DbNotExist.into())
        }
    }
}

impl MallocSizeOf for SingleMptStorageManager {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut size = 0;
        size += self.node_memory_manager.size_of(ops);
        size
    }
}
