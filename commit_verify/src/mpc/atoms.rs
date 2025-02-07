// Client-side-validation foundation libraries.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2023 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2023 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::Write;

use amplify::confinement::MediumOrdMap;
use amplify::num::u5;
use amplify::{Bytes32, FromSliceError, Wrapper};

use crate::id::CommitmentId;
use crate::merkle::MerkleNode;
use crate::{strategies, CommitEncode, CommitStrategy};

pub const MPC_MINIMAL_DEPTH: u5 = u5::with(3);

/// Map from protocol ids to commitment messages.
pub type MessageMap = MediumOrdMap<ProtocolId, Message>;

/// Source data for creation of multi-message commitments according to [LNPBP-4]
/// procedure.
///
/// [LNPBP-4]: https://github.com/LNP-BP/LNPBPs/blob/master/lnpbp-0004.md
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From, Default)]
#[wrapper(Deref, BorrowSlice, Display, FromStr, Hex, Index, RangeOps)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = crate::LIB_NAME_COMMIT_VERIFY)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct ProtocolId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl CommitStrategy for ProtocolId {
    type Strategy = strategies::Strict;
}

impl ProtocolId {
    pub fn copy_from_slice(slice: &[u8]) -> Result<Self, FromSliceError> {
        Bytes32::copy_from_slice(slice).map(Self)
    }
}

/// Original message participating in multi-message commitment.
///
/// The message must be represented by a 32-byte hash.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From, Default)]
#[wrapper(Deref, BorrowSlice, Display, FromStr, Hex, Index, RangeOps)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = crate::LIB_NAME_COMMIT_VERIFY)]
#[derive(CommitEncode)]
#[commit_encode(crate = crate, strategy = strict)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Message(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl Message {
    pub fn copy_from_slice(slice: &[u8]) -> Result<Self, FromSliceError> {
        Bytes32::copy_from_slice(slice).map(Self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
pub enum Leaf {
    Inhabited {
        protocol: ProtocolId,
        message: Message,
    },
    Entropy {
        entropy: u64,
        pos: u32,
    },
}

impl Leaf {
    pub fn entropy(entropy: u64, pos: u32) -> Self { Self::Entropy { entropy, pos } }

    pub fn inhabited(protocol: ProtocolId, message: Message) -> Self {
        Self::Inhabited { protocol, message }
    }
}

impl CommitEncode for Leaf {
    fn commit_encode(&self, e: &mut impl Write) {
        match self {
            Leaf::Inhabited { protocol, message } => {
                // We use this constant since we'd like to be distinct from NodeBranching values
                0x10.commit_encode(e);
                protocol.commit_encode(e);
                message.commit_encode(e);
            }
            Leaf::Entropy { entropy, pos } => {
                // We use this constant since we'd like to be distinct from NodeBranching values
                0x11.commit_encode(e);
                entropy.commit_encode(e);
                pos.commit_encode(e);
            }
        }
    }
}

impl CommitmentId for Leaf {
    const TAG: [u8; 32] = *b"urn:lnpbp:lnpbp0004:leaf:v01#23A";
    type Id = MerkleNode;
}

/// Final [LNPBP-4] commitment value.
///
/// Represents tagged hash of the merkle root of [`super::MerkleTree`] and
/// [`super::MerkleBlock`].
///
/// [LNPBP-4]: https://github.com/LNP-BP/LNPBPs/blob/master/lnpbp-0004.md
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Display, FromStr, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = crate::LIB_NAME_COMMIT_VERIFY)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Commitment(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl CommitStrategy for Commitment {
    type Strategy = strategies::Strict;
}

impl Commitment {
    pub fn copy_from_slice(slice: &[u8]) -> Result<Self, FromSliceError> {
        Bytes32::copy_from_slice(slice).map(Self)
    }
}

/// Structured source multi-message data for commitment creation
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MultiSource {
    /// Minimal depth of the created LNPBP-4 commitment tree
    pub min_depth: u5,
    /// Map of the messages by their respective protocol ids
    pub messages: MessageMap,
    pub static_entropy: Option<u64>,
}

impl Default for MultiSource {
    #[inline]
    fn default() -> Self {
        MultiSource {
            min_depth: MPC_MINIMAL_DEPTH,
            messages: Default::default(),
            static_entropy: None,
        }
    }
}

impl MultiSource {
    #[inline]
    pub fn with_static_entropy(static_entropy: u64) -> Self {
        MultiSource {
            static_entropy: Some(static_entropy),
            ..default!()
        }
    }
}
