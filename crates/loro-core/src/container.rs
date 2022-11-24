//! CRDT [Container]. Each container may have different CRDT type [ContainerType].
//! Each [Op] has an associated container. It's the [Container]'s responsibility to
//! calculate the state from the [Op]s.
//!
//! Every [Container] can take a [Snapshot], which contains [crate::LoroValue] that describes the state.
//!
use crate::{
    event::{Observer, SubscriptionID},
    hierarchy::Hierarchy,
    log_store::ImportContext,
    op::{InnerContent, RemoteContent, RichOp},
    version::{IdSpanVector, VersionVector},
    InternalString, LogStore, LoroValue, ID,
};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use std::{any::Any, fmt::Debug};

pub mod registry;

pub mod list;
pub mod map;
mod pool;
pub mod text;

#[cfg_attr(feature = "test_utils", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ContainerType {
    /// See [`crate::text::TextContent`]
    Text,
    Map,
    List,
    // TODO: Users can define their own container types.
    // Custom(u16),
}

pub trait Container: Debug + Any + Unpin {
    fn id(&self) -> &ContainerID;
    fn type_(&self) -> ContainerType;
    fn get_value(&self) -> LoroValue;
    // TODO: need a custom serializer
    // fn serialize(&self) -> Vec<u8>;

    /// convert an op content to exported format that includes the raw data
    fn to_export(&mut self, content: InnerContent, gc: bool) -> SmallVec<[RemoteContent; 1]>;

    /// convert an op content to compact imported format
    fn to_import(&mut self, content: RemoteContent) -> InnerContent;

    /// Tracker need to retreat in order to apply the op.
    /// TODO: can be merged into checkout
    fn track_retreat(&mut self, spans: &IdSpanVector);

    /// Tracker need to forward in order to apply the op.
    /// TODO: can be merged into checkout
    fn track_forward(&mut self, spans: &IdSpanVector);

    /// Tracker need to checkout to target version in order to apply the op.
    fn tracker_checkout(&mut self, vv: &VersionVector);

    /// Apply the op to the tracker.
    ///
    /// Here we have not updated the container state yet. Because we
    /// need to calculate the effect of the op for [crate::List] and
    /// [crate::Text] by using tracker.  
    fn track_apply(
        &mut self,
        hierarchy: &mut Hierarchy,
        op: &RichOp,
        import_context: &mut ImportContext,
    );

    /// Apply the effect of the op directly to the state.
    fn update_state_directly(
        &mut self,
        hierarchy: &mut Hierarchy,
        op: &RichOp,
        import_context: &mut ImportContext,
    );
    /// Make tracker iterate over the target spans and apply the calculated
    /// effects to the container state
    fn apply_tracked_effects_from(
        &mut self,
        store: &mut LogStore,
        import_context: &mut ImportContext,
    );
    fn subscribe(
        &self,
        hierarchy: &mut Hierarchy,
        observer: Observer,
        deep: bool,
    ) -> SubscriptionID {
        hierarchy.subscribe(self.id(), observer, deep)
    }

    fn unsubscribe(&self, hierarchy: &mut Hierarchy, subscription: SubscriptionID) {
        hierarchy.unsubscribe(self.id(), subscription);
    }
}

/// [ContainerID] includes the Op's [ID] and the type. So it's impossible to have
/// the same [ContainerID] with conflict [ContainerType].
///
/// This structure is really cheap to clone
#[derive(Hash, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ContainerID {
    /// Root container does not need a insert op to create. It can be created implicitly.
    Root {
        name: InternalString,
        container_type: ContainerType,
    },
    Normal {
        id: ID,
        container_type: ContainerType,
    },
}

pub enum ContainerIdRaw {
    Root { name: InternalString },
    Normal { id: ID },
}

impl From<&str> for ContainerIdRaw {
    fn from(s: &str) -> Self {
        ContainerIdRaw::Root { name: s.into() }
    }
}

impl From<ID> for ContainerIdRaw {
    fn from(id: ID) -> Self {
        ContainerIdRaw::Normal { id }
    }
}

impl From<&ContainerID> for ContainerIdRaw {
    fn from(id: &ContainerID) -> Self {
        match id {
            ContainerID::Root { name, .. } => ContainerIdRaw::Root { name: name.clone() },
            ContainerID::Normal { id, .. } => ContainerIdRaw::Normal { id: *id },
        }
    }
}

impl From<ContainerID> for ContainerIdRaw {
    fn from(id: ContainerID) -> Self {
        match id {
            ContainerID::Root { name, .. } => ContainerIdRaw::Root { name },
            ContainerID::Normal { id, .. } => ContainerIdRaw::Normal { id },
        }
    }
}

impl ContainerIdRaw {
    pub fn with_type(self, container_type: ContainerType) -> ContainerID {
        match self {
            ContainerIdRaw::Root { name } => ContainerID::Root {
                name,
                container_type,
            },
            ContainerIdRaw::Normal { id } => ContainerID::Normal { id, container_type },
        }
    }
}

impl ContainerID {
    #[inline]
    pub fn new_normal(id: ID, container_type: ContainerType) -> Self {
        ContainerID::Normal { id, container_type }
    }

    #[inline]
    pub fn new_root(name: &str, container_type: ContainerType) -> Self {
        ContainerID::Root {
            name: name.into(),
            container_type,
        }
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        matches!(self, ContainerID::Root { .. })
    }

    #[inline]
    pub fn is_normal(&self) -> bool {
        matches!(self, ContainerID::Normal { .. })
    }

    #[inline]
    pub fn name(&self) -> &InternalString {
        match self {
            ContainerID::Root { name, .. } => name,
            ContainerID::Normal { .. } => unreachable!(),
        }
    }

    #[inline]
    pub fn container_type(&self) -> ContainerType {
        match self {
            ContainerID::Root { container_type, .. } => *container_type,
            ContainerID::Normal { container_type, .. } => *container_type,
        }
    }
}
