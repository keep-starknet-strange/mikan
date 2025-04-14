use malachitebft_core_types::Round;

use crate::{context::TestContext, height::Height, value::Value};
use malachitebft_test::Address;
/// A proposal for a value in a round
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub height: Height,
    pub round: Round,
    pub value: Value,
    pub pol_round: Round,
    pub validator_address: Address,
}


impl Proposal {
    pub fn new(
        height: Height,
        round: Round,
        value: Value,
        pol_round: Round,
        validator_address: Address,
    ) -> Self {
        Self {
            height,
            round,
            value,
            pol_round,
            validator_address,
        }
    }
}

impl malachitebft_core_types::Proposal<TestContext> for Proposal {
    fn height(&self) -> Height {
        self.height
    }

    fn round(&self) -> Round {
        self.round
    }

    fn value(&self) -> &Value {
        &self.value
    }

    fn take_value(self) -> Value {
        self.value
    }

    fn pol_round(&self) -> Round {
        self.pol_round
    }

    fn validator_address(&self) -> &Address {
        &self.validator_address
    }
}
