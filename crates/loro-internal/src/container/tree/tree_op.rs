use fractional_index::FractionalIndex;
use loro_common::TreeID;
use rle::{HasLength, Mergable};
use serde::{Deserialize, Serialize};

use crate::state::TreeParentId;

/// The operation of movable tree.
///
/// In the movable tree, there are three actions:
/// - **Create**: target tree id will be generated by [`Transaction`], and parent tree id is `None`.
/// - **Move**: move target tree node a child node of the specified parent node.
/// - **Delete**: move target tree node to [`loro_common::DELETED_TREE_ROOT`].
///
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreeOp {
    Create {
        target: TreeID,
        parent: Option<TreeID>,
        position: FractionalIndex,
    },
    Move {
        target: TreeID,
        parent: Option<TreeID>,
        position: FractionalIndex,
    },
    Delete {
        target: TreeID,
    },
}

impl TreeOp {
    pub(crate) fn target(&self) -> TreeID {
        match self {
            TreeOp::Create { target, .. } => *target,
            TreeOp::Move { target, .. } => *target,
            TreeOp::Delete { target, .. } => *target,
        }
    }

    pub(crate) fn parent(&self) -> Option<TreeID> {
        match self {
            TreeOp::Create { parent, .. } => *parent,
            TreeOp::Move { parent, .. } => *parent,
            TreeOp::Delete { .. } => Some(TreeID::delete_root()),
        }
    }

    pub(crate) fn parent_id(&self) -> TreeParentId {
        match self {
            TreeOp::Create { parent, .. } => TreeParentId::from(*parent),
            TreeOp::Move { parent, .. } => TreeParentId::from(*parent),
            TreeOp::Delete { .. } => TreeParentId::Deleted,
        }
    }

    pub(crate) fn fractional_index(&self) -> Option<FractionalIndex> {
        match self {
            TreeOp::Create { position, .. } | TreeOp::Move { position, .. } => {
                Some(position.clone())
            }
            TreeOp::Delete { .. } => None,
        }
    }
}

impl HasLength for TreeOp {
    fn content_len(&self) -> usize {
        1
    }
}

impl Mergable for TreeOp {}
