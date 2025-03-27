use malachitebft_core_types::{NilOrVal, Round, SignedExtension, VoteType};

use crate::address::Address;
use crate::context::TestContext;
use crate::height::Height;
use crate::value::ValueId;

pub use malachitebft_core_types::Extension;

/// A vote for a value in a round
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vote {
    pub typ: VoteType,
    pub height: Height,
    pub round: Round,
    pub value: NilOrVal<ValueId>,
    pub validator_address: Address,
    pub extension: Option<SignedExtension<TestContext>>,
}

impl Vote {
    pub fn new_prevote(
        height: Height,
        round: Round,
        value: NilOrVal<ValueId>,
        validator_address: Address,
    ) -> Self {
        Self {
            typ: VoteType::Prevote,
            height,
            round,
            value,
            validator_address,
            extension: None,
        }
    }

    pub fn new_precommit(
        height: Height,
        round: Round,
        value: NilOrVal<ValueId>,
        address: Address,
    ) -> Self {
        Self {
            typ: VoteType::Precommit,
            height,
            round,
            value,
            validator_address: address,
            extension: None,
        }
    }
}

impl malachitebft_core_types::Vote<TestContext> for Vote {
    fn height(&self) -> Height {
        self.height
    }

    fn round(&self) -> Round {
        self.round
    }

    fn value(&self) -> &NilOrVal<ValueId> {
        &self.value
    }

    fn take_value(self) -> NilOrVal<ValueId> {
        self.value
    }

    fn vote_type(&self) -> VoteType {
        self.typ
    }

    fn validator_address(&self) -> &Address {
        &self.validator_address
    }

    fn extension(&self) -> Option<&SignedExtension<TestContext>> {
        self.extension.as_ref()
    }

    fn take_extension(&mut self) -> Option<SignedExtension<TestContext>> {
        self.extension.take()
    }

    fn extend(self, extension: SignedExtension<TestContext>) -> Self {
        Self {
            extension: Some(extension),
            ..self
        }
    }
}
