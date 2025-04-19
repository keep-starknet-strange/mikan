//! Internal state of the application. This is a simplified abstract to keep it simple.
//! A regular application would have mempool implemented, a proper database and input methods like RPC.

use crate::block::Block;
use crate::malachite_types::codec::proto::ProtobufCodec;
use crate::malachite_types::signing::Ed25519Provider;
use crate::malachite_types::value::Value;
use crate::malachite_types::{
    address::Address,
    context::TestContext,
    genesis::Genesis,
    height::Height,
    proposal_part::{ProposalData, ProposalFin, ProposalInit, ProposalPart},
    validator_set::ValidatorSet,
};
use crate::store::{DecidedValue, Store};
use crate::streaming::{PartStreamsMap, ProposalParts};
use crate::transactions::pool::TransactionPool;
use bincode::config::standard;
use bytes::Bytes;
use chrono::Utc;
use color_eyre::eyre;
use eyre::Result;
use malachitebft_app_channel::app::streaming::{StreamContent, StreamId, StreamMessage};
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_app_channel::app::types::core::{CommitCertificate, Round, Validity};
use malachitebft_app_channel::app::types::{LocallyProposedValue, PeerId, ProposedValue};
use sha3::Digest;
use std::collections::HashSet;
use std::mem::size_of;
use tracing::{debug, error, info};

/// Size of chunks in which the data is split for streaming
const CHUNK_SIZE: usize = 128 * 1024; // 128 KiB

// Path to the file containing the genesis
// const GENESIS_PATH: &str = "./data/genesis.json";

/// Maximum number of blocks to keep in history
const MAX_HISTORY_LENGTH: u64 = 25;

/// Represents the internal state of the application node
/// Contains information about current height, round, proposals and blocks
pub struct State {
    _ctx: TestContext,
    genesis: Genesis,
    signing_provider: Ed25519Provider,
    address: Address,
    pub store: Store,
    stream_nonce: u32,
    streams_map: PartStreamsMap,
    // block_proposer: BlockProposer,
    // block_executor: BlockExecutor,
    // rpc_server: Option<RpcServerHandle>,
    // TODO: replace this with rpc server
    pub transaction_pool: TransactionPool,
    pub current_height: Height,
    pub current_round: Round,
    pub current_proposer: Option<Address>,
    pub peers: HashSet<PeerId>,
}

/// Represents errors that can occur during the verification of a proposal's signature.
#[derive(Debug)]
enum SignatureVerificationError {
    /// Indicates that the `Fin` part of the proposal is missing.
    MissingFinPart,

    /// Indicates that the proposer was not found in the validator set.
    ProposerNotFound,

    /// Indicates that the signature in the `Fin` part is invalid.
    InvalidSignature,
}

impl State {
    #[allow(clippy::too_many_arguments)]
    /// Creates a new State instance with the given validator address and starting height
    pub async fn new(
        genesis: Genesis,
        ctx: TestContext,
        signing_provider: Ed25519Provider,
        address: Address,
        height: Height,
        store: Store,
        transaction_pool: TransactionPool,
        _enable_rpc: bool,
    ) -> Self {
        // Get the node's home directory from the store path
        let store_path = store.get_path();
        let node_dir = store_path.parent().unwrap().parent().unwrap();
        let _db_path = node_dir.join("mikan_db");

        // Extract node index from the directory name

        let node_index = node_dir
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse::<usize>().ok())
            .expect("Node directory should be a number");

        let _blocks_file = format!("./data/blocks-{}", node_index);

        // let eth_genesis_json = std::fs::read_to_string(ETH_GENESIS_PATH).unwrap();
        // let eth_genesis: EthGenesis = serde_json::from_str(&eth_genesis_json).unwrap();

        // let block_executor = BlockExecutor::new(db_path, eth_genesis.clone()).unwrap();
        // let rpc_server = if enable_rpc {
        //     match block_executor.start_server().await {
        //         Ok(handle) => {
        //             info!("RPC server started successfully");
        //             Some(handle)
        //         }
        //         Err(e) => {
        //             error!("Failed to start RPC server: {}", e);
        //             None
        //         }
        //     }
        // } else {
        //     None
        // };

        Self {
            genesis,
            _ctx: ctx,
            signing_provider,
            current_height: height,
            current_round: Round::new(0),
            current_proposer: None,
            address,
            store,
            stream_nonce: 0,
            streams_map: PartStreamsMap::new(),
            peers: HashSet::new(),
            transaction_pool,
            // block_proposer: BlockProposer::new(&blocks_file).unwrap(),
            // block_executor,
            // rpc_server,
        }
    }

    pub async fn make_block(&mut self) -> eyre::Result<Bytes> {
        let prev_block = self
            .store
            .get_decided_block(self.current_height - 1)
            .await?;
        let prev_block = prev_block.unwrap();
        let (prev_block, _): (Block, usize) =
            bincode::borrow_decode_from_slice(prev_block.as_ref(), standard())?;

        let mut tx = self.transaction_pool.get_top_transaction();
        while !tx.validate() {
            error!("Invalid transaction, skipping");
            tx = self.transaction_pool.get_top_transaction();
        }
        info!(
            "Valid transaction, {} adding to block",
            hex::encode(tx.hash())
        );
        let block = Block::new(
            self.current_height.as_u64(),
            Utc::now().timestamp() as u64,
            prev_block.hash(),
            self.address,
            vec![tx],
        );

        let block_data = bincode::encode_to_vec(&block, standard())?;
        Ok(Bytes::from(block_data))
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

        // Check if we have a full proposal - for now we are assuming that the network layer will stop spam/DOS
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

        if let Err(e) = self.verify_proposal_signature(&parts) {
            error!(
                height = %self.current_height,
                round = %self.current_round,
                error = ?e,
                "Received proposal with invalid signature, ignoring"
            );

            return Ok(None);
        }

        let part_height = parts.height;
        let part_round = parts.round;

        // Re-assemble the proposal from its parts
        let (value, data) = assemble_value_from_parts(parts);
        let (block, _): (Block, usize) = bincode::borrow_decode_from_slice(&data, standard())?;
        let prev_block = self
            .store
            .get_decided_block(self.current_height - 1)
            .await?;
        let Some(prev_block) = prev_block else {
            error!("Previous block not found");
            return Ok(None);
        };
        let (prev_block, _) = bincode::borrow_decode_from_slice(prev_block.as_ref(), standard())?;
        if !block.is_valid(self.current_height.as_u64(), &prev_block)? {
            error!("Invalid block");
            return Ok(None);
        }

        // Log first 32 bytes of proposal data and total size
        if data.len() >= 32 {
            info!(
                "Proposal data[0..32]: {}, total_size: {} bytes, id: {:x}",
                hex::encode(&data[..32]),
                data.len(),
                value.value.id().as_u64()
            );
        }

        // Store the proposal and its data
        self.store.store_undecided_proposal(value.clone()).await?;
        self.store
            .store_undecided_block_data(part_height, part_round, data)
            .await?;

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
    ) -> eyre::Result<()> {
        info!(
            height = %certificate.height,
            round = %certificate.round,
            "Looking for certificate"
        );

        let proposal = self
            .store
            .get_undecided_proposal(certificate.height, certificate.round)
            .await;

        let proposal = match proposal {
            Ok(Some(proposal)) => proposal,
            Ok(None) => {
                error!(
                    height = %certificate.height,
                    round = %certificate.round,
                    "Trying to commit a value that is not decided"
                );

                return Ok(()); // FIXME: Return an actual error and handle in caller
            }
            Err(e) => return Err(e.into()),
        };

        self.store
            .store_decided_value(&certificate, proposal.value)
            .await?;

        // Store block data for decided value
        let block_data = self
            .store
            .get_block_data(certificate.height, certificate.round)
            .await?;

        if let Some(data) = block_data {
            self.store
                .store_decided_block_data(certificate.height, data.clone())
                .await?;

            // Only execute blocks if this node is running the RPC server
            if !data.is_empty()
            // && self.rpc_server.is_some() rpc is not implemented yet
            {
                // Execute the block in the background
                // let executor = self.block_executor.clone();
                // let height = certificate.height;
                // tokio::task::spawn_blocking(move || match executor.next_block(&data) {
                //     Ok(_) => info!(height = %height, "Successfully executed block"),
                //     Err(e) => {
                //         error!(height = %height, "Failed to execute block: {}. Continuing with consensus...", e)
                //     }
                // });
            }
        }

        // Prune the store
        let retain_height = Height::new(
            certificate
                .height
                .as_u64()
                .saturating_sub(MAX_HISTORY_LENGTH),
        );
        self.store.prune(retain_height).await?;

        // Move to next height
        self.current_height = self.current_height.increment();
        self.current_round = Round::new(0);

        Ok(())
    }

    /// Creates a new proposal value for the given height
    /// Returns either a previously built proposal or creates a new one
    pub async fn propose_value(
        &mut self,
        height: Height,
        round: Round,
        data: Bytes,
    ) -> eyre::Result<LocallyProposedValue<TestContext>> {
        assert_eq!(height, self.current_height);
        assert_eq!(round, self.current_round);

        // We create a new value.
        let value = Value::new(data.clone()); // Clone the data since we need it twice

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

        // Also store the block data
        self.store
            .store_undecided_block_data(height, round, data)
            .await?;

        Ok(LocallyProposedValue::new(
            proposal.height,
            proposal.round,
            proposal.value,
        ))
    }

    fn stream_id(&mut self) -> StreamId {
        let mut bytes = Vec::with_capacity(size_of::<u64>() + size_of::<u32>());
        bytes.extend_from_slice(&self.current_height.as_u64().to_be_bytes());
        bytes.extend_from_slice(&self.current_round.as_u32().unwrap().to_be_bytes());
        bytes.extend_from_slice(&self.stream_nonce.to_be_bytes());
        self.stream_nonce += 1;
        StreamId::new(bytes.into())
    }

    /// Creates a stream message containing a proposal part.
    /// Updates internal sequence number and current proposal.
    pub fn stream_proposal(
        &mut self,
        value: LocallyProposedValue<TestContext>,
        data: Bytes,
    ) -> impl Iterator<Item = StreamMessage<ProposalPart>> {
        let parts = self.make_proposal_parts(value, data);

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

    fn make_proposal_parts(
        &self,
        value: LocallyProposedValue<TestContext>,
        data: Bytes,
    ) -> Vec<ProposalPart> {
        let mut hasher = sha3::Keccak256::new();
        let mut parts = Vec::new();

        // Init
        {
            parts.push(ProposalPart::Init(ProposalInit::new(
                value.height,
                value.round,
                self.address,
            )));

            hasher.update(value.height.as_u64().to_be_bytes().as_slice());
            hasher.update(value.round.as_i64().to_be_bytes().as_slice());
        }

        // Data
        {
            for chunk in data.chunks(CHUNK_SIZE) {
                let chunk_data = ProposalData::new(Bytes::copy_from_slice(chunk));
                parts.push(ProposalPart::Data(chunk_data));
                hasher.update(chunk);
            }
        }

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
        let mut signature = None;

        // Recreate the hash and extract the signature during traversal
        for part in &parts.parts {
            match part {
                ProposalPart::Init(init) => {
                    hasher.update(init.height.as_u64().to_be_bytes());
                    hasher.update(init.round.as_i64().to_be_bytes());
                }
                ProposalPart::Data(data) => {
                    hasher.update(data.bytes.as_ref());
                }
                ProposalPart::Fin(fin) => {
                    signature = Some(&fin.signature);
                }
            }
        }

        let hash = hasher.finalize();
        let signature = signature.ok_or(SignatureVerificationError::MissingFinPart)?;

        // Retrieve the public key of the proposer
        let public_key = self
            .get_validator_set()
            .get_by_address(&parts.proposer)
            .map(|v| v.public_key);

        let public_key = public_key.ok_or(SignatureVerificationError::ProposerNotFound)?;

        // Verify the signature
        if !self.signing_provider.verify(&hash, signature, &public_key) {
            return Err(SignatureVerificationError::InvalidSignature);
        }

        Ok(())
    }
}

/// Re-assemble a [`ProposedValue`] from its [`ProposalParts`].
fn assemble_value_from_parts(parts: ProposalParts) -> (ProposedValue<TestContext>, Bytes) {
    // Calculate total size and allocate buffer
    let total_size: usize = parts
        .parts
        .iter()
        .filter_map(|part| part.as_data())
        .map(|data| data.bytes.len())
        .sum();

    let mut data = Vec::with_capacity(total_size);
    // Concatenate all chunks
    for part in parts.parts.iter().filter_map(|part| part.as_data()) {
        data.extend_from_slice(&part.bytes);
    }

    // Convert the concatenated data vector into Bytes
    let data = Bytes::from(data);

    let proposed_value = ProposedValue {
        height: parts.height,
        round: parts.round,
        valid_round: Round::Nil,
        proposer: parts.proposer,
        value: Value::new(data.clone()),
        validity: Validity::Valid,
    };

    (proposed_value, data)
}

/// Decodes a Value from its byte representation using ProtobufCodec
pub fn decode_value(bytes: Bytes) -> Value {
    ProtobufCodec.decode(bytes).unwrap()
}
