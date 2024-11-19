# Smart Contract Source Code Retriever

## Description

This is a command-line tool written in Rust for retrieving smart contract source code from various blockchains. It supports multiple chains, including Ethereum and Binance Smart Chain, and can batch read contract addresses and chain information from a CSV file.

## Features

- Retrieve smart contract source code from specified blockchains
- Support for multiple blockchain networks
- Batch processing of contract information from CSV files
- Customizable output directory
- Supported chains: eth, bsc, ftm, mumbai, pg, avax, rinkeby, goerli, arb, op, sepolia, base, boba-ethereum, boba-bnb, boba-avax, moonbeam, moonriver, cro, rsk, zora, merlin, pg-amoy, bitlayer, mode, scroll

## Installation

Ensure you have Rust and Cargo installed on your system. Then, follow these steps to install:

```bash
https://github.com/Kong-F/Smart-Contract-Source-Code-Retriever.git
cd Smart-Contract-Source-Code-Retriever
# Configure the required block browser api key in the `main.rs` file
cargo build --release
```

## Usage

```bash
USAGE:
    smart_contract_retriever [OPTIONS]

OPTIONS:
    -c, --chain <CHAIN>        Specify the chain (required in single mode)
    -d, --address <ADDRESS>    Specify the address (required in single mode)
    -f, --file <FILE>          Specify the file (required in batch mode). e.g. 0x0,eth
    -h, --help                 Print help information
    -l, --list                 List all supported chains
    -o, --output <OUTPUT>      Specify the output directory [default: ./output]
    -V, --version              Print version information
```

Basic usage:

```bash
./smart_contract_retriever -o ./output_directory -f contracts.csv
./smart_contract_retriever -o ./output_directory -d 0x00... -c eth
```

Parameters:
- `-o, --output`: Specify the output directory (optional, default is "./output")
- `-f, --file`: Specify the CSV file containing contract addresses and chain information
- `-d, --address`: Specifies the smart contract address where you want to get the open source code
- `-c --chain`: Specifies the chain on which the smart contract resides

CSV file format:
```
contract_address,chain_name
0x1234...,eth
0x5678...,bsc
```

## Contributing

Issues and pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)