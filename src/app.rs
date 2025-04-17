use color_eyre::eyre::{self, eyre};
use std::time::Duration;
use tracing::{error, info};

use crate::malachite_types::codec::proto::ProtobufCodec;
use crate::malachite_types::context::TestContext;
use crate::state::{decode_value, State};
use malachitebft_app_channel::app::streaming::StreamContent;
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_app_channel::app::types::core::{Round, Validity};
use malachitebft_app_channel::app::types::sync::RawDecidedValue;
use malachitebft_app_channel::app::types::ProposedValue;
use malachitebft_app_channel::{AppMsg, Channels, ConsensusMsg, NetworkMsg};

pub async fn run(state: &mut State, channels: &mut Channels<TestContext>) -> eyre::Result<()> {
    while let Some(msg) = channels.consensus.recv().await {
        match msg {
            // The first message to handle is the `ConsensusReady` message, signaling to the app
            // that Malachite is ready to start consensus
            AppMsg::ConsensusReady { reply } => {
                info!("Consensus is ready");

                tokio::time::sleep(Duration::from_millis(500)).await;

                // We can simply respond by telling the engine to start consensus
                // at the current height, which is initially 1
                if reply
                    .send((state.current_height, state.get_validator_set().clone()))
                    .is_err()
                {
                    error!("Failed to send ConsensusReady reply");
                }
            }

            // The next message to handle is the `StartRound` message, signaling to the app
            // that consensus has entered a new round (including the initial round 0)
            AppMsg::StartedRound {
                height,
                round,
                proposer,
                reply_value,
            } => {
                info!(%height, %round, %proposer, "Started round");

                // We can use that opportunity to update our internal state
                state.current_height = height;
                state.current_round = round;
                state.current_proposer = Some(proposer);

                // If we have already built or seen a value for this height and round,
                // send it back to consensus. This may happen when we are restarting after a crash.
                if let Some(proposal) = state.store.get_undecided_proposal(height, round).await? {
                    info!(%height, %round, "Replaying already known proposed value: {}", proposal.value.id());

                    if reply_value.send(Some(proposal)).is_err() {
                        error!("Failed to send undecided proposal");
                    }
                } else {
                    let _ = reply_value.send(None);
                }
            }

            // At some point, we may end up being the proposer for that round, and the engine
            // will then ask us for a value to propose to the other validators.
            AppMsg::GetValue {
                height,
                round,
                timeout: _,
                reply,
            } => {
                // NOTE: We can ignore the timeout as we are building the value right away.
                // If we were let's say reaping as many txes from a mempool and executing them,
                // then we would need to respect the timeout and stop at a certain point.

                info!(%height, %round, "Consensus is requesting a value to propose");

                // We need to create a new value to propose and send it back to consensus.
                // Get block data
                let block_bytes = state.make_block().await?;

                let proposal = state
                    .propose_value(height, round, block_bytes.clone())
                    .await?;

                // Send it to consensus
                if reply.send(proposal.clone()).is_err() {
                    error!("Failed to send GetValue reply");
                }

                // Now what's left to do is to break down the value to propose into parts,
                // and send those parts over the network to our peers, for them to re-assemble the full value.
                for stream_message in state.stream_proposal(proposal, block_bytes) {
                    info!(%height, %round, "Streaming proposal part: {stream_message:?}");

                    channels
                        .network
                        .send(NetworkMsg::PublishProposalPart(stream_message))
                        .await?;
                }
            }

            // On the receiving end of these proposal parts (ie. when we are not the proposer),
            // we need to process these parts and re-assemble the full value.
            // To this end, we store each part that we receive and assemble the full value once we
            // have all its constituent parts. Then we send that value back to consensus for it to
            // consider and vote for or against it (ie. vote `nil`), depending on its validity.
            AppMsg::ReceivedProposalPart { from, part, reply } => {
                let (part_type, part_size) = match &part.content {
                    StreamContent::Data(part) => (part.get_type(), part.size_bytes()),
                    StreamContent::Fin => ("end of stream", 0),
                };

                info!(
                    %from, %part.sequence, part.type = %part_type, part.size = %part_size,
                    "Received proposal part"
                );

                let proposed_value = state.received_proposal_part(from, part).await?;

                if reply.send(proposed_value).is_err() {
                    error!("Failed to send ReceivedProposalPart reply");
                }
            }

            // In some cases, e.g. to verify the signature of a vote received at a higher height
            // than the one we are at (e.g. because we are lagging behind a little bit),
            // the engine may ask us for the validator set at that height.
            //
            // In our case, our validator set stays constant between heights so we can
            // send back the validator set found in our genesis state.
            AppMsg::GetValidatorSet { height: _, reply } => {
                if reply.send(state.get_validator_set().clone()).is_err() {
                    error!("Failed to send GetValidatorSet reply");
                }
            }

            // After some time, consensus will finally reach a decision on the value
            // to commit for the current height, and will notify the application,
            // providing it with a commit certificate which contains the ID of the value
            // that was decided on as well as the set of commits for that value,
            // ie. the precommits together with their (aggregated) signatures.
            AppMsg::Decided {
                certificate, reply, ..
            } => {
                let height = certificate.height;
                let round = certificate.round;
                let value_id = certificate.value_id;

                info!(
                    height = %height, round = %round, value = %value_id,
                    "Consensus has decided on value"
                );

                // When that happens, we store the decided value in our store
                info!(height = %height, round = %round, value = %value_id, "Committing decided value");
                state.commit(certificate).await?;
                info!(height = %height, round = %round, value = %value_id, "Committed decided value");

                // Pause briefly before starting next height, just to make following the logs easier
                // tokio::time::sleep(Duration::from_millis(500)).await;

                // And then we instruct consensus to start the next height
                if reply
                    .send(ConsensusMsg::StartHeight(
                        state.current_height,
                        state.get_validator_set().clone(),
                    ))
                    .is_err()
                {
                    error!("Failed to send Decided reply");
                }
            }

            // It may happen that our node is lagging behind its peers. In that case,
            // a synchronization mechanism will automatically kick to try and catch up to
            // our peers. When that happens, some of these peers will send us decided values
            // for the heights in between the one we are currently at (included) and the one
            // that they are at. When the engine receives such a value, it will forward to the application
            // to decode it from its wire format and send back the decoded value to consensus.
            //
            // TODO: store the received value somewhere here
            AppMsg::ProcessSyncedValue {
                height,
                round,
                proposer,
                value_bytes,
                reply,
            } => {
                info!(%height, %round, "Processing synced value");

                let value = decode_value(value_bytes);

                // We send to consensus to see if it has been decided on
                if reply
                    .send(ProposedValue {
                        height,
                        round,
                        valid_round: Round::Nil,
                        proposer,
                        value,
                        validity: Validity::Valid,
                    })
                    .is_err()
                {
                    error!("Failed to send ProcessSyncedValue reply");
                }
            }

            // If, on the other hand, we are not lagging behind but are instead asked by one of
            // our peer to help them catch up because they are the one lagging behind,
            // then the engine might ask the application to provide with the value
            // that was decided at some lower height. In that case, we fetch it from our store
            // and send it to consensus.
            AppMsg::GetDecidedValue { height, reply } => {
                let decided_value = state.get_decided_value(height).await;

                let raw_decided_value = decided_value.map(|decided_value| RawDecidedValue {
                    certificate: decided_value.certificate,
                    value_bytes: ProtobufCodec.encode(&decided_value.value).unwrap(),
                });

                if reply.send(raw_decided_value).is_err() {
                    error!("Failed to send GetDecidedValue reply");
                }
            }

            // In order to figure out if we can help a peer that is lagging behind,
            // the engine may ask us for the height of the earliest available value in our store.
            AppMsg::GetHistoryMinHeight { reply } => {
                let min_height = state.get_earliest_height().await;

                if reply.send(min_height).is_err() {
                    error!("Failed to send GetHistoryMinHeight reply");
                }
            }

            AppMsg::RestreamProposal { .. } => {
                error!("RestreamProposal not implemented");
            }

            AppMsg::ExtendVote { reply, .. } => {
                if reply.send(None).is_err() {
                    error!("Failed to send ExtendVote reply");
                }
            }

            AppMsg::VerifyVoteExtension { reply, .. } => {
                if reply.send(Ok(())).is_err() {
                    error!("Failed to send VerifyVoteExtension reply");
                }
            }

            AppMsg::PeerJoined { peer_id } => {
                info!(%peer_id, "Peer joined our local view of network");

                // You might want to track connected peers in your state
                state.peers.insert(peer_id);
            }

            AppMsg::PeerLeft { peer_id } => {
                info!(%peer_id, "Peer left our local view of network");

                // Remove the peer from tracking
                state.peers.remove(&peer_id);
            }
        }
    }

    // If we get there, it can only be because the channel we use to receive message
    // from consensus has been closed, meaning that the consensus actor has died.
    // We can do nothing but return an error here.
    Err(eyre!("Consensus channel closed unexpectedly"))
}
