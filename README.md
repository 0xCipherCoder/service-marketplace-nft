# Service Marketplace NFT

This project implements a 2-sided marketplace model for services using Anchor, a framework for Solana programs. Vendors can list services, with service agreements represented by metadata in NFTs. Consumers can purchase these service NFTs. The marketplace supports both soulbound and non-soulbound NFTs and collects royalties on resales.

## Scope

Develop a program using Anchor to create a 2-sided marketplace for services. Vendors should be able to list services as NFTs, and consumers should be able to purchase them. The marketplace should support both soulbound and non-soulbound NFTs, with royalty collection on resales.

## Features

1. **Service Listing**: Allows vendors to list services as NFTs.
2. **Service Purchase**: Enables consumers to purchase service NFTs.
3. **Soulbound and Non-Soulbound NFTs**: Supports both types of NFTs.
4. **Royalty Collection**: Collects royalties on the resale of service NFTs.

## Prerequisites

- Rust
- Solana CLI
- Node.js
- Yarn
- Anchor

## Installation

1. **Install Solana CLI**:
    ```sh
    sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
    ```

2. **Install Rust**:
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

3. **Install Node.js and Yarn**:
    ```sh
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
    nvm install --lts
    npm install --global yarn
    ```

4. **Install Anchor**:
    ```sh
    cargo install --git https://github.com/project-serum/anchor --tag v0.19.0 anchor-cli --locked
    ```

## Setup

1. Clone the repository:
    ```sh
    git clone git@github.com:0xCipherCoder/service-marketplace-nft.git
    cd service-marketplace-nft
    ```

2. Install dependencies:
    ```sh
    npm install
    anchor build
    ```

3. Deploy the programs to Solana Local Testnet:
    ```sh
    anchor deploy
    ```

## Usage

### Building the Program

1. Build the Solana program:
    ```sh
    anchor build
    ```

2. Deploy the program to your local Solana cluster:
    ```sh
    anchor deploy
    ```

### Running Tests

1. Run the tests:
    ```sh
    anchor test
    ```
2. Ensure your local Solana test validator is running - Not required for testing:
    ```sh
    solana-test-validator
    ```
    

### Test Report

```sh
anchor test
    Finished release [optimized] target(s) in 0.09s

Found a 'test' script in the Anchor.toml. Running it as a test suite!

Running test suite: "/home/pradip/Cipher/OpenSource/marketplace/Anchor.toml"
  service-marketplace
    ✔ Initializes the marketplace (429ms)
    ✔ Lists a service (428ms)
    ✔ Purchases a service (869ms)
    ✔ Resells a service (1734ms)
