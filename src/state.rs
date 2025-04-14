//! Internal state of the application. This is a simplified abstract to keep it simple.
//! A regular application would have mempool implemented, a proper database and input methods like RPC.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::block::Block;
use bytes::{Bytes, BytesMut};
use eyre::eyre;
use sha3::Digest;
use tokio::time::sleep;
use tracing::{debug, error, info};

use malachitebft_app_channel::app::consensus::ProposedValue;
use malachitebft_app_channel::app::streaming::{StreamContent, StreamId, StreamMessage};
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_app_channel::app::types::core::{
    CommitCertificate, Round, Validity, VoteExtensions,
};
use malachitebft_app_channel::app::types::{LocallyProposedValue, PeerId};
use malachitebft_test::codec::proto::ProtobufCodec;
use malachitebft_test::{
    Address, Ed25519Provider, Genesis, Height, ProposalData, ProposalFin, ProposalInit,
    ProposalPart, TestContext, ValidatorSet, Value,
};

use crate::store::{DecidedValue, Store};
use crate::streaming::{PartStreamsMap, ProposalParts};

/// Number of historical values to keep in the store
const HISTORY_LENGTH: u64 = 100;

/// Represents the internal state of the application node
/// Contains information about current height, round, proposals and blocks
pub struct State {
    #[allow(dead_code)]
    ctx: TestContext,
    signing_provider: Ed25519Provider,
    genesis: Genesis,
    address: Address,
    vote_extensions: HashMap<Height, VoteExtensions<TestContext>>,
    streams_map: PartStreamsMap,

    pub store: Store,
    pub current_height: Height,
    pub current_round: Round,
    pub current_proposer: Option<Address>,
    pub peers: HashSet<PeerId>,
    pub block: Block,
}

/// Represents errors that can occur during the verification of a proposal's signature.
#[derive(Debug)]
enum SignatureVerificationError {
    /// Indicates that the `Init` part of the proposal is unexpectedly missing.
    MissingInitPart,

    /// Indicates that the `Fin` part of the proposal is unexpectedly missing.
    MissingFinPart,

    /// Indicates that the proposer was not found in the validator set.
    ProposerNotFound,

    /// Indicates that the signature in the `Fin` part is invalid.
    InvalidSignature,
}

impl State {
    /// Creates a new State instance with the given validator address and starting height
    pub fn new(
        ctx: TestContext,
        signing_provider: Ed25519Provider,
        genesis: Genesis,
        address: Address,
        height: Height,
        store: Store,
        block: Block,
    ) -> Self {
        Self {
            ctx,
            signing_provider,
            genesis,
            current_height: height,
            current_round: Round::new(0),
            current_proposer: None,
            address,
            store,
            vote_extensions: HashMap::new(),
            streams_map: PartStreamsMap::new(),
            peers: HashSet::new(),
            block,
        }
    }

    /// Returns the earliest height available in the state
    pub async fn get_earliest_height(&self) -> Height {
        self.store
            .min_decided_value_height()
            .await
            .unwrap_or_default()
    }

    /// Processes and adds a new proposal to the state if it's valid
    /// Returns Some(ProposedValue) if the proposal was accepted, None otherwise
    pub async fn received_proposal_part(
        &mut self,
        from: PeerId,
        part: StreamMessage<ProposalPart>,
    ) -> eyre::Result<Option<ProposedValue<TestContext>>> {
        let sequence = part.sequence;

        // Check if we have a full proposal
        let Some(parts) = self.streams_map.insert(from, part) else {
            return Ok(None);
        };

        // Check if the proposal is outdated
        if parts.height < self.current_height {
            debug!(
                height = %self.current_height,
                round = %self.current_round,
                part.height = %parts.height,
                part.round = %parts.round,
                part.sequence = %sequence,
                "Received outdated proposal part, ignoring"
            );

            return Ok(None);
        }

        // Verify the proposal signature
        match self.verify_proposal_signature(&parts) {
            Ok(()) => {
                // Signature verified successfully, continue processing
            }
            Err(SignatureVerificationError::MissingInitPart) => {
                return Err(eyre!(
                    "Expected to have full proposal but `Init` proposal part is missing for proposer: {}",
                    parts.proposer
                ));
            }
            Err(SignatureVerificationError::MissingFinPart) => {
                return Err(eyre!(
                    "Expected to have full proposal but `Fin` proposal part is missing for proposer: {}",
                    parts.proposer
                ));
            }
            Err(SignatureVerificationError::ProposerNotFound) => {
                error!(proposer = %parts.proposer, "Proposer not found in validator set");
                return Ok(None);
            }
            Err(SignatureVerificationError::InvalidSignature) => {
                error!(proposer = %parts.proposer, "Invalid signature in Fin part");
                return Ok(None);
            }
        }

        // Re-assemble the proposal from its parts
        let value = assemble_value_from_parts(parts)?;

        info!(
            "Storing undecided proposal {} {}",
            value.height, value.round
        );

        self.store.store_undecided_proposal(value.clone()).await?;

        Ok(Some(value))
    }

    /// Retrieves a decided block at the given height
    pub async fn get_decided_value(&self, height: Height) -> Option<DecidedValue> {
        self.store.get_decided_value(height).await.ok().flatten()
    }

    /// Commits a value with the given certificate, updating internal state
    /// and moving to the next height
    pub async fn commit(
        &mut self,
        certificate: CommitCertificate<TestContext>,
        extensions: VoteExtensions<TestContext>,
    ) -> eyre::Result<()> {
        let (height, round, value_id) =
            (certificate.height, certificate.round, certificate.value_id);

        // Store extensions for use at next height if we are the proposer
        self.vote_extensions.insert(height.increment(), extensions);

        // Get the first proposal with the given value id. There may be multiple identical ones
        // if peers have restreamed at different rounds.
        let Ok(Some(proposal)) = self
            .store
            .get_undecided_proposal_by_value_id(value_id)
            .await
        else {
            return Err(eyre!(
                "Trying to commit a value with value id {value_id} at height {height} and round {round} for which there is no proposal"
            ));
        };

        self.store
            .store_decided_value(&certificate, proposal.value)
            .await?;

        // Remove all proposals with the given value id.
        self.store
            .remove_undecided_proposals_by_value_id(value_id)
            .await?;

        // Prune the store, keep the last HISTORY_LENGTH values
        let retain_height = Height::new(height.as_u64().saturating_sub(HISTORY_LENGTH));
        self.store.prune(retain_height).await?;

        // Move to next height
        self.current_height = self.current_height.increment();
        self.current_round = Round::new(0);

        Ok(())
    }

    /// Retrieves a previously built proposal value for the given height
    pub async fn get_previously_built_value(
        &self,
        height: Height,
        round: Round,
    ) -> eyre::Result<Option<LocallyProposedValue<TestContext>>> {
        let Some(proposal) = self.store.get_undecided_proposal(height, round).await? else {
            return Ok(None);
        };

        Ok(Some(LocallyProposedValue::new(
            proposal.height,
            proposal.round,
            proposal.value,
        )))
    }

    /// Creates a new proposal value for the given height
    /// Returns either a previously built proposal or creates a new one
    async fn create_proposal(
        &mut self,
        height: Height,
        round: Round,
    ) -> eyre::Result<ProposedValue<TestContext>> {
        assert_eq!(height, self.current_height);
        assert_eq!(round, self.current_round);

        // We create a new value.
        let value = self.make_value(height, round);

        // Simulate some processing time
        sleep(Duration::from_millis(500)).await;

        let proposal = ProposedValue {
            height,
            round,
            valid_round: Round::Nil,
            proposer: self.address, // We are the proposer
            value,
            validity: Validity::Valid, // Our proposals are de facto valid
        };

        // Insert the new proposal into the undecided proposals.
        self.store
            .store_undecided_proposal(proposal.clone())
            .await?;

        Ok(proposal)
    }

    /// Make up a new value to propose
    /// A real application would have a more complex logic here,
    /// typically reaping transactions from a mempool and executing them against its state,
    /// before computing the merkle root of the new app state.
    fn make_value(&mut self, height: Height, _round: Round) -> Value {
        let block_hash = self.block.header.block_hash.clone();
        let value = u64::from_be_bytes(block_hash.as_slice().try_into().unwrap_or_default());

        // TODO: Where should we verify signatures?
        let extensions = self
            .vote_extensions
            .remove(&height)
            .unwrap_or_default()
            .extensions
            .into_iter()
            .map(|(_, e)| e.message)
            .fold(BytesMut::new(), |mut acc, e| {
                acc.extend_from_slice(&e);
                acc
            })
            .freeze();

        Value { value, extensions }
    }

    /// Creates a new proposal value for the given height
    /// Returns either a previously built proposal or creates a new one
    pub async fn propose_value(
        &mut self,
        height: Height,
        round: Round,
    ) -> eyre::Result<LocallyProposedValue<TestContext>> {
        assert_eq!(height, self.current_height);
        assert_eq!(round, self.current_round);

        let proposal = self.create_proposal(height, round).await?;

        Ok(LocallyProposedValue::new(
            proposal.height,
            proposal.round,
            proposal.value,
        ))
    }

    /// Creates a stream message containing a proposal part.
    /// Updates internal sequence number and current proposal.
    pub fn stream_proposal(
        &mut self,
        value: LocallyProposedValue<TestContext>,
        pol_round: Round,
    ) -> impl Iterator<Item = StreamMessage<ProposalPart>> {
        let parts = self.value_to_parts(value, pol_round);
        let stream_id = self.stream_id();

        let mut msgs = Vec::with_capacity(parts.len() + 1);
        let mut sequence = 0;

        for part in parts {
            let msg = StreamMessage::new(stream_id.clone(), sequence, StreamContent::Data(part));
            sequence += 1;
            msgs.push(msg);
        }

        msgs.push(StreamMessage::new(stream_id, sequence, StreamContent::Fin));

        msgs.into_iter()
    }

    fn stream_id(&self) -> StreamId {
        let mut bytes = Vec::with_capacity(size_of::<u64>() + size_of::<u32>());
        bytes.extend_from_slice(&self.current_height.as_u64().to_be_bytes());
        bytes.extend_from_slice(&self.current_round.as_u32().unwrap().to_be_bytes());
        StreamId::new(bytes.into())
    }

    fn value_to_parts(
        &self,
        value: LocallyProposedValue<TestContext>,
        pol_round: Round,
    ) -> Vec<ProposalPart> {
        let mut hasher = sha3::Keccak256::new();
        let mut parts = Vec::new();

        // Init
        // Include metadata about the proposal
        {
            parts.push(ProposalPart::Init(ProposalInit::new(
                value.height,
                value.round,
                pol_round,
                self.address,
            )));

            hasher.update(value.height.as_u64().to_be_bytes().as_slice());
            hasher.update(value.round.as_i64().to_be_bytes().as_slice());
        }

        // Data
        // Include each prime factor of the value as a separate proposal part
        {
            for factor in factor_value(value.value) {
                parts.push(ProposalPart::Data(ProposalData::new(factor)));

                hasher.update(factor.to_be_bytes().as_slice());
            }
        }

        // Fin
        // Sign the hash of the proposal parts
        {
            let hash = hasher.finalize().to_vec();
            let signature = self.signing_provider.sign(&hash);
            parts.push(ProposalPart::Fin(ProposalFin::new(signature)));
        }

        parts
    }

    /// Returns the set of validators.
    pub fn get_validator_set(&self) -> &ValidatorSet {
        &self.genesis.validator_set
    }

    /// Verifies the signature of the proposal.
    /// Returns `Ok(())` if the signature is valid, or an appropriate `SignatureVerificationError`.
    fn verify_proposal_signature(
        &self,
        parts: &ProposalParts,
    ) -> Result<(), SignatureVerificationError> {
        let mut hasher = sha3::Keccak256::new();

        let init = parts
            .init()
            .ok_or(SignatureVerificationError::MissingInitPart)?;

        let fin = parts
            .fin()
            .ok_or(SignatureVerificationError::MissingFinPart)?;

        let hash = {
            hasher.update(init.height.as_u64().to_be_bytes());
            hasher.update(init.round.as_i64().to_be_bytes());

            // The correctness of the hash computation relies on the parts being ordered by sequence
            // number, which is guaranteed by the `PartStreamsMap`.
            for part in parts.parts.iter().filter_map(|part| part.as_data()) {
                hasher.update(part.factor.to_be_bytes());
            }

            hasher.finalize()
        };

        // Retrieve the the proposer
        let proposer = self
            .get_validator_set()
            .get_by_address(&parts.proposer)
            .ok_or(SignatureVerificationError::ProposerNotFound)?;

        // Verify the signature
        if !self
            .signing_provider
            .verify(&hash, &fin.signature, &proposer.public_key)
        {
            return Err(SignatureVerificationError::InvalidSignature);
        }

        Ok(())
    }
}

/// Re-assemble a [`ProposedValue`] from its [`ProposalParts`].
///
/// This is done by multiplying all the factors in the parts.
fn assemble_value_from_parts(parts: ProposalParts) -> eyre::Result<ProposedValue<TestContext>> {
    let init = parts.init().ok_or_else(|| eyre!("Missing Init part"))?;

    let value = parts
        .parts
        .iter()
        .filter_map(|part| part.as_data())
        .fold(1, |acc, data| acc * data.factor);

    Ok(ProposedValue {
        height: parts.height,
        round: parts.round,
        valid_round: init.pol_round,
        proposer: parts.proposer,
        value: Value::new(value),
        validity: Validity::Valid,
    })
}

/// Decodes a Value from its byte representation using ProtobufCodec
pub fn decode_value(bytes: Bytes) -> Value {
    ProtobufCodec.decode(bytes).unwrap()
}

/// Returns the list of prime factors of the given value
///
/// In a real application, this would typically split transactions
/// into chunks ino order to reduce bandwidth requirements due
/// to duplication of gossip messages.
fn factor_value(value: Value) -> Vec<u64> {
    let mut factors = Vec::new();
    let mut n = value.value;

    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            factors.push(i);
            n /= i;
        } else {
            i += 1;
        }
    }

    if n > 1 {
        factors.push(n);
    }

    factors
}
