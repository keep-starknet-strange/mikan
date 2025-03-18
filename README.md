<div align="center">

<a href="https://github.com/AbdelStark/bitcoin-mcp/actions/workflows/ci.yml"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/AbdelStark/mikan/ci.yml?style=for-the-badge" height=30></a>
<a href="https://bitcoin.org/"> <img alt="Bitcoin" src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white" height=30></a>

</div>

# Mikan 🍊: The ZK Friendly DA Layer for Bitcoin L2s

<div align="center">
  <h3>
    <a href="https://github.com/AbdelStark/mikan">
      DOCS
    </a>
    <span> | </span>
    <a href="https://eprint.iacr.org/2024/248">
      FRIDA PAPER
    </a>
    <span> | </span>
    <a href="https://github.com/keep-starknet-strange/frieda">
      FRIEDA LIB
    </a>
  </h3>
</div>

Mikan is a ZK-Rollup friendly data availability layer built on the Malachite consensus framework.
It provides strong data availability guarantees using STARK-friendly cryptographic primitives
without requiring a trusted setup.

It is design to be particularly friendly to ZK-Rollups, specifically on Bitcoin, by providing a DA layer that is
compatible with the FRI commitment scheme, and by eliminating the need for a trusted setup.

> Name origin 🍊 (蜜柑):
>
> Officially known as a Citrus unshiu or unshiu mikan, mikan is a type of mandarin orange and citrus fruit that originated in southern Japan approximately 400 years ago.

## 💼 Table of Contents

- [Mikan 🍊: The ZK Friendly DA Layer for Bitcoin L2s](#mikan--the-zk-friendly-da-layer-for-bitcoin-l2s)
  - [💼 Table of Contents](#-table-of-contents)
  - [🔧 Key Features](#-key-features)
  - [📐 Architecture](#-architecture)
    - [🔑 Cryptographic Components](#-cryptographic-components)
  - [🎮 Getting Started](#-getting-started)
    - [Prerequisites](#prerequisites)
    - [Run a local testnet](#run-a-local-testnet)
      - [Build the app](#build-the-app)
    - [Setup the testnet](#setup-the-testnet)
    - [Spawn the nodes](#spawn-the-nodes)
  - [Usage for Rollups](#usage-for-rollups)
  - [🤝 Contributing](#-contributing)
  - [🗺️ Roadmap](#️-roadmap)
  - [📝 License](#-license)
  - [📚 References](#-references)

## 🔧 Key Features

- **No Trusted Setup**: Unlike solutions using KZG commitments, Mikan relies on transparent
  cryptography based on the FRI protocol, eliminating the need for trusted setup ceremonies.

- **ZK-Friendly Proofs**: Mikan generates succinct proofs of data availability using the M31
  field (Mersenne prime 2^31-1), which is STARK-friendly and can be efficiently verified
  inside ZK proofs.

- **Fast Consensus**: Built on the Malachite BFT consensus framework, which provides
  high throughput and quick finality.

## 📐 Architecture

Mikan consists of the following components:

1. **Consensus Layer**: Based on Malachite BFT, handling block production and agreement.
2. **Data Availability Layer**: Uses FRI commitments and proofs for data availability sampling.
3. **Network Layer**: Ensures quick propagation of proposal parts across the network.
4. **API**: Allows rollups to submit data and retrieve availability certificates.

### 🔑 Cryptographic Components

- **FRI Protocol**: Fast Reed-Solomon Interactive Oracle Proof for data availability.
- **M31 Field**: Uses the Mersenne prime 2^31-1 for efficient field arithmetic.
- **Merkle Trees**: For efficient commitments and proofs.

The core primitives for the Data Availability Sampling are implemented in [FRI Extended for Data Availability: a FRI-based Data Availability Sampling library, written in Rust.](https://github.com/keep-starknet-strange/frieda).

## 🎮 Getting Started

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

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🗺️ Roadmap

See [ROADMAP.md](ROADMAP.md).

## 📝 License

This project is licensed under the [MIT License](LICENSE).

## 📚 References

- [FRI Extended for Data Availability: a FRI-based Data Availability Sampling library, written in Rust.](https://github.com/keep-starknet-strange/frieda)
- [FRIDA: Data Availability Sampling from FRI](https://eprint.iacr.org/2024/248)
- [A Guide to Selecting the Right Data Availability Layer](https://blog.availproject.org/a-guide-to-selecting-the-right-data-availability-layer/)
- [M31 arithmetic opcodes for efficient STARK verification on Bitcoin](https://hackmd.io/@abdelhamid/m31-opcodes-bitcoin-stark)
- [Malachite - Flexible BFT consensus engine in Rust](https://github.com/informalsystems/malachite)
