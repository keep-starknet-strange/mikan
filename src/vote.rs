use crate::address::Address;
use crate::context::TestContext;
use crate::height::Height;
use crate::value::ValueId;
use malachitebft_core_types::{NilOrVal, Round, SignedExtension, VoteType};

#[allow(dead_code)]
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Vote {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block: usize,
    pub height: Height,
    pub round: Round,
    pub typ: VoteType,
    pub validator_address: Address,
    pub value: NilOrVal<ValueId>,
    pub extension: Option<SignedExtension<TestContext>>,
}

#[allow(dead_code)]
impl Vote {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        validator: Address,
        sig: Vec<u8>,
        block: usize,
        height: Height,
        round: Round,
        typ: VoteType,
        validator_address: Address,
        value: NilOrVal<ValueId>,
        extension: Option<SignedExtension<TestContext>>,
    ) -> Self {
        Self {
            validator,
            signature: sig,
            block,
            height,
            round,
            typ,
            validator_address,
            value,
            extension,
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
