// LNP/BP Rust Library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use num_traits::{FromPrimitive, ToPrimitive};
use std::collections::BTreeMap;
use std::io;

/// For now, Simplicity script is not implemented, so we use a byte array as a
/// placeholder for script data
pub type SimplicityScript = Vec<u8>;

/// Marker trait for all node-specific action types
pub trait NodeAction: ToPrimitive + FromPrimitive + Ord
where
    Self: Sized,
{
}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    ToPrimitive,
    FromPrimitive,
)]
#[display(Debug)]
pub enum GenesisAction {}
impl NodeAction for GenesisAction {}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    ToPrimitive,
    FromPrimitive,
)]
#[display(Debug)]
pub enum ExtensionAction {}
impl NodeAction for ExtensionAction {}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    ToPrimitive,
    FromPrimitive,
)]
#[display(Debug)]
#[repr(u16)]
pub enum TransitionAction {
    GenerateBlank = 0,
}
impl NodeAction for TransitionAction {}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    ToPrimitive,
    FromPrimitive,
)]
#[display(Debug)]
#[repr(u16)]
pub enum AssignmentAction {
    Validate = 0,
}

pub type GenesisAbi = BTreeMap<GenesisAction, Procedure>;
pub type ExtensionAbi = BTreeMap<ExtensionAction, Procedure>;
pub type TransitionAbi = BTreeMap<TransitionAction, Procedure>;
pub type AssignmentAbi = BTreeMap<AssignmentAction, Procedure>;

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Display)]
#[display(Debug)]
pub enum Procedure {
    Standard(StandardProcedure),
    Simplicity { offset: u32 },
}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    ToPrimitive,
    FromPrimitive,
)]
#[display(Debug)]
#[repr(u8)]
pub enum StandardProcedure {
    ConfidentialAmount = 1,
    IssueControl = 2,
    Prunning = 3,
}

mod strict_encoding {
    use super::*;
    use crate::strict_encoding::{Error, StrictDecode, StrictEncode};

    impl_enum_strict_encoding!(GenesisAction);
    impl_enum_strict_encoding!(TransitionAction);
    impl_enum_strict_encoding!(AssignmentAction);
    impl_enum_strict_encoding!(ExtensionAction);

    impl_enum_strict_encoding!(StandardProcedure);

    impl StrictEncode for Procedure {
        type Error = Error;

        fn strict_encode<E: io::Write>(
            &self,
            mut e: E,
        ) -> Result<usize, Error> {
            Ok(match self {
                Self::Simplicity { offset } => {
                    strict_encode_list!(e; 0u8, offset)
                }
                Self::Standard(proc_id) => {
                    strict_encode_list!(e; 0xFFu8, proc_id)
                }
            })
        }
    }

    impl StrictDecode for Procedure {
        type Error = Error;

        fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Error> {
            Ok(match u8::strict_decode(&mut d)? {
                0u8 => Self::Simplicity {
                    offset: u32::strict_decode(&mut d)?,
                },
                0xFFu8 => {
                    Self::Standard(StandardProcedure::strict_decode(&mut d)?)
                }
                x => Err(Error::EnumValueNotKnown(
                    "script::Procedure".to_string(),
                    x,
                ))?,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::strict_encoding::strict_encode;

        #[test]
        fn test_basics() {
            // Test Actions and Standard procedures
            test_enum_u8_exhaustive!(AssignmentAction; AssignmentAction::Validate => 0);
            test_enum_u8_exhaustive!(TransitionAction; TransitionAction::GenerateBlank => 0);
            test_enum_u8_exhaustive!(StandardProcedure; StandardProcedure::ConfidentialAmount => 1, 
                StandardProcedure::IssueControl => 2, 
                StandardProcedure::Prunning => 3);

            // Test Procedures
            assert_eq!(
                vec![0xFF, 0x01],
                strict_encode(&Procedure::Standard(
                    StandardProcedure::ConfidentialAmount
                ))
                .unwrap()
            );
            assert_eq!(
                vec![0xFF, 0x02],
                strict_encode(&Procedure::Standard(
                    StandardProcedure::IssueControl
                ))
                .unwrap()
            );
            assert_eq!(
                vec![0xFF, 0x03],
                strict_encode(&Procedure::Standard(
                    StandardProcedure::Prunning
                ))
                .unwrap()
            );
            assert_eq!(
                vec![0x00, 0x58, 0x00, 0x00, 0x00],
                strict_encode(&Procedure::Simplicity { offset: 88 }).unwrap()
            );

            // Test Transition and Assignment ABI
            let mut trans_abi = TransitionAbi::new();
            trans_abi.insert(
                TransitionAction::GenerateBlank,
                Procedure::Standard(StandardProcedure::ConfidentialAmount),
            );
            assert_eq!(
                vec![0x01, 0x00, 0x00, 0xff, 0x01],
                strict_encode(&trans_abi).unwrap()
            );

            let mut assignment_abi = AssignmentAbi::new();
            assignment_abi.insert(
                AssignmentAction::Validate,
                Procedure::Simplicity { offset: 45 },
            );
            assert_eq!(
                vec![0x01, 0x00, 0x00, 0x00, 0x2d, 0x00, 0x00, 0x00],
                strict_encode(&assignment_abi).unwrap()
            );
        }
    }
}
