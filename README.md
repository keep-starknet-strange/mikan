# Mikan 🍊: The ZK Friendly DA Layer for Bitcoin L2s

Mikan is a ZK-Rollup friendly data availability layer built on the Malachite consensus framework.
It provides strong data availability guarantees using STARK-friendly cryptographic primitives
without requiring a trusted setup.

It is design to be particularly friendly to ZK-Rollups, specifically on Bitcoin, by providing a DA layer that is
compatible with the FRI commitment scheme, and by eliminating the need for a trusted setup.

> Name origin 🍊 (蜜柑):
>
> Officially known as a Citrus unshiu or unshiu mikan, mikan is a type of mandarin orange and citrus fruit that originated in southern Japan approximately 400 years ago.

## Key Features

- **No Trusted Setup**: Unlike solutions using KZG commitments, Mikan relies on transparent
  cryptography based on the FRI protocol, eliminating the need for trusted setup ceremonies.

- **ZK-Friendly Proofs**: Mikan generates succinct proofs of data availability using the M31
  field (Mersenne prime 2^31-1), which is STARK-friendly and can be efficiently verified
  inside ZK proofs.

- **Fast Consensus**: Built on the Malachite BFT consensus framework, which provides
  high throughput and quick finality.

- **Erasure Coding**: Data is erasure-coded with a 4x expansion factor, allowing data
  availability to be verified by sampling just a small fraction of the total data.

## Architecture

Mikan consists of the following components:

1. **Consensus Layer**: Based on Malachite BFT, handling block production and agreement.
2. **Data Availability Layer**: Uses FRI commitments and proofs for data availability sampling.
3. **Network Layer**: Ensures quick propagation of proposal parts across the network.
4. **API**: Allows rollups to submit data and retrieve availability certificates.

### Cryptographic Components

- **FRI Protocol**: Fast Reed-Solomon Interactive Oracle Proof for data availability.
- **M31 Field**: Uses the Mersenne prime 2^31-1 for efficient field arithmetic.
- **Merkle Trees**: For efficient commitments and proofs.

## Getting Started

### Prerequisites

- Rust 1.71.0 or later
- Cargo

### Run a local testnet

#### Build the app

```bash
cargo build
```

### Setup the testnet

Generate configuration and genesis for three nodes using the `testnet` command:

```bash
cargo run -- testnet --nodes 3 --home nodes
```

This will create the configuration for three nodes in the `nodes` folder. Feel free to inspect this folder and look at the generated files.

### Spawn the nodes

```bash
bash spawn.bash --nodes 3 --home nodes
```

If successful, the logs for each node can then be found at `nodes/X/logs/node.log`.

```bash
tail -f nodes/0/logs/node.log
```

Press `Ctrl-C` to stop all the nodes.

## Usage for Rollups

Rollups can submit data to Mikan via its API:

1. **Submit Data**: Rollup submits transaction data to Mikan nodes
2. **Receive Commitment**: Mikan returns a cryptographic commitment to the data
3. **Verify Availability**: Anyone can verify data availability by sampling

The FRI commitment can be included in the rollup's state transition proof, creating
a seamless integration between the rollup's validity proofs and data availability guarantees.

## Contributing

Contributions are welcome\! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [FRIDA: Data Availability Sampling from FRI](https://eprint.iacr.org/2024/248)
- [A Guide to Selecting the Right Data Availability Layer](https://blog.availproject.org/a-guide-to-selecting-the-right-data-availability-layer/)
- [M31 arithmetic opcodes for efficient STARK verification on Bitcoin](https://hackmd.io/@abdelhamid/m31-opcodes-bitcoin-stark)
- [Malachite - Flexible BFT consensus engine in Rust](https://github.com/informalsystems/malachite)
