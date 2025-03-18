# Mikan Development Roadmap

This roadmap outlines the incremental development plan to build Mikan, a ZK-Rollup friendly Data Availability layer without trusted setup.

## Current System

- A basic Malachite consensus app where the state is an in-memory random integer
- The node is capable of running a decentralized network, synchronizing, and reaching consensus

## Target System

- The consensus-agreed "value" becomes a "block" containing transaction blobs
- Integration with the frieda library for FRI-based commitments and proofs
- ZK-friendly DA layer using M31 field and FRI commitments without trusted setup

## Key Architectural Components

- **Consensus Layer:** Built on Malachite BFT
- **Data Availability Layer:** New block format with DA proofs using frieda
- **Storage Layer:** Persistent storage for blocks and DA commitments
- **Network Layer:** Block parts and DA information dissemination

## Development Milestones

### Milestone 1: Basic Block Support with Dummy DA Integration

**Goals:**

- Replace random integer values with blocks containing transaction blobs
- Basic frieda integration for commitment computation
- Consensus engine proposing, disseminating, and committing blocks

**Key Issues:**

- #1: Define and implement block data structure
- #2: Integrate basic frieda wrapper for commitments
- #3: Update proposal logic for block-based values
- #4: Modify storage for block persistence
- #5: Test local network consensus on blocks

### Milestone 2: Validator DA Proof Sampling and Verification

**Goals:**

- Extend frieda integration to include data sampling
- Enable validators to verify block availability via DA sampling

**Key Issues:**

- #6: Implement frieda sampling API integration
- #7: Add validator DA sampling logic before voting
- #8: Extend proposal format for DA proof data
- #9: Add tests for invalid block rejection

### Milestone 3: Full End-to-End Integration and API Support

**Goals:**

- Complete DA layer functionality
- Provide APIs for rollups to submit blobs and query DA certificates

**Key Issues:**

- #10: Finalize frieda integration for full proof generation/verification
- #11: Create API endpoints for rollup interaction
- #12: Conduct end-to-end testnet validation
- #13: Benchmark performance metrics

### Milestone 4: Code Cleanup, Documentation, and Production Hardening

**Goals:**

- Clean separation of DA, consensus, and application layers
- Comprehensive documentation and usage instructions

**Key Issues:**

- #14: Refactor codebase for clarity and maintainability
- #15: Update documentation with DA feature explanations
- #16: Implement DA-specific logging and monitoring
- #17: Security and performance review

## Testing Strategy

- **Unit Tests:** Frieda wrapper functions, block serialization/deserialization
- **Integration Tests:** Multi-node testnet simulation, DA integrity validation
- **Benchmarking:** Throughput and latency measurements
- **Documentation:** Updated setup and running instructions

This roadmap provides a structured approach to transform the current basic consensus system into a full-featured Data Availability layer for ZK-Rollups.
