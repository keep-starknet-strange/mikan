use bytes::Bytes;
use malachitebft_core_types::{Context, NilOrVal, Round, ValidatorSet as _};
use malachitebft_test::Ed25519;

use crate::address::*;
use crate::height::*;
use crate::proposal::*;
use crate::proposal_part::*;
use crate::validator::*;
use crate::value::*;
use crate::vote::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct TestContext;

impl TestContext {
    pub fn new() -> Self {
        Self
    }
}

impl Context for TestContext {
    type Address = Address;
    type ProposalPart = ProposalPart;
    type Height = Height;
    type Proposal = Proposal;
    type ValidatorSet = ValidatorSet;
    type Validator = Validator;
    type Value = Value;
    type Vote = Vote;
    type SigningScheme = Ed25519;
    type Extension = Bytes;

    fn select_proposer<'a>(
        &self,
        validator_set: &'a Self::ValidatorSet,
        height: Self::Height,
        round: Round,
    ) -> &'a Self::Validator {
        assert!(validator_set.count() > 0);
        assert!(round != Round::Nil && round.as_i64() >= 0);

        let proposer_index = {
            let height = height.as_u64() as usize;
            let round = round.as_i64() as usize;

            (height - 1 + round) % validator_set.count()
        };

        validator_set
            .get_by_index(proposer_index)
            .expect("proposer_index is valid")
    }

    fn new_proposal(
        _height: Height,
        _round: Round,
        _value: Value,
        _pol_round: Round,
        _address: Address,
    ) -> Proposal {
        unimplemented!()
    }

    fn new_prevote(
        _height: Self::Height,
        _round: Round,
        _value_id: NilOrVal<ValueId>,
        _address: Self::Address,
    ) -> Self::Vote {
        unimplemented!()
    }

    fn new_precommit(
        _height: Self::Height,
        _round: Round,
        _value_id: NilOrVal<ValueId>,
        _address: Self::Address,
    ) -> Self::Vote {
        unimplemented!()
    }
}
