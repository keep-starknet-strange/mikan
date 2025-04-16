use super::address::Address;
use bytes::Bytes;
use malachitebft_core_types::{Context, NilOrVal, Round, ValidatorSet as _};
use malachitebft_test::Ed25519;

use super::height::Height;
use super::proposal::Proposal;
use super::proposal_part::ProposalPart;
use super::validator_set::Validator;
use super::validator_set::ValidatorSet;
use super::value::Value;
use super::vote::Vote;

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
        &self,
        height: Self::Height,
        round: Round,
        value: Self::Value,
        pol_round: Round,
        address: Self::Address,
    ) -> Self::Proposal {
        Proposal::new(height, round, value, pol_round, address)
    }

    fn new_prevote(
        &self,
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<malachitebft_core_types::ValueId<Self>>,
        address: Self::Address,
    ) -> Self::Vote {
        Vote::new_prevote(height, round, value_id, address)
    }

    fn new_precommit(
        &self,
        height: Self::Height,
        round: Round,
        value_id: NilOrVal<malachitebft_core_types::ValueId<Self>>,
        address: Self::Address,
    ) -> Self::Vote {
        Vote::new_precommit(height, round, value_id, address)
    }
}
