#[cfg(feature="prusti")]
use prusti_contracts::*;
use std::{collections::BTreeMap, fmt};

use ibc::ics24_host::identifier::ChainId;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    object::{Object, ObjectType},
    worker::WorkerId,
};

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Serialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
pub struct WorkerDesc {
    pub id: WorkerId,
    pub object: Object,
}

impl WorkerDesc {
    pub fn new(id: WorkerId, object: Object) -> Self {
        Self { id, object }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
pub struct SupervisorState {
    pub chains: Vec<ChainId>,
    pub workers: BTreeMap<ObjectType, Vec<WorkerDesc>>,
}

impl SupervisorState {
    #[cfg(feature="prusti")]
    #[trusted]
    pub fn new<'a>(
        mut chains: Vec<ChainId>,
        workers: impl Iterator<Item = (WorkerId, &'a Object)>,
    ) -> Self {
        todo!()
    }

    #[cfg(not(feature="prusti"))]
    pub fn new<'a>(
        mut chains: Vec<ChainId>,
        workers: impl Iterator<Item = (WorkerId, &'a Object)>,
    ) -> Self {
        chains.sort();

        let workers = workers
            .map(|(id, o)| WorkerDesc::new(id, o.clone()))
            .into_group_map_by(|desc| desc.object.object_type())
            .into_iter()
            .update(|(_, os)| os.sort_by_key(|desc| desc.object.short_name()))
            .collect::<BTreeMap<_, _>>();

        Self { chains, workers }
    }

    #[cfg(not(feature="prusti"))]
    pub fn print_info(&self) {
        self.to_string()
            .split('\n')
            .for_each(|line| info!("{}", line));
    }
    #[cfg(feature="prusti")]
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn print_info(&self) { }
}

impl fmt::Display for SupervisorState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "* Chains: {}", self.chains.iter().join(", "))?;
        for (tpe, objects) in &self.workers {
            writeln!(f, "* {:?} workers:", tpe)?;
            for desc in objects {
                writeln!(f, "  - {} (id: {})", desc.object.short_name(), desc.id)?;
            }
        }

        Ok(())
    }
}
