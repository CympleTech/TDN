[English](./README.md) / [中文](./README_zh.md)

- Start Date: 2019-06-03
- RFC PR #0
- TDN Issue #0

Summary
=======

Design a trusted distributed network that allows everyone to manage their data freely and securely. Developers can implement interactive, computable, serverless applications based on this network, while helping to communicate and data between applications.

Motivation
==========

There are a variety of applications in the world, there are various needs, so we can not strictly limit the data structure, account system, consensus algorithm, communication algorithm, and propagation algorithm of trusted distributed systems. We only limit the applications communication, the data sharing mechanism between applications, and how to achieve more stable and reliable consistent results between applications. Therefore, the core library of our project will encapsulate various cumbersome technical details and inter-application Communication and data specifications make the necessary constraints, and will gradually provide P2P network libraries, distributed storage libraries, virtual machine libraries, distributed computing libraries, and libraries for private data access and privacy calculations. Developers can make according to their own needs. The trade-off is therefore made up of a series of infrastructure libraries.


Design
===============

### Design Concept
- Support different data structures
- Support different consensus algorithms
- Support different permission mechanisms
- Support different account systems
- Applications can communicate with others
- Trust can be passed on and accumulated

[Design Detail RFC](https://github.com/placefortea/TDN/blob/master/rfcs/1_design.md)

### Core Library
#### - [Teatree](https://github.com/placefortea/teatree) - [RFC](https://github.com/placefortea/TDN/blob/master/rfcs/2_architecture_and_infrastructure.md)
Architecture implementation and infrastructure libraries, including network library, communication library, storage library, cryptography library, primitive types, and constraint libraries.

#### - [Black Tea](https://github.com/placefortea/black_tea) - [RFC](https://github.com/placefortea/TDN/blob/master/rfcs/3_distributed_storage.md)
Pluggable distributed storage library, as a library to provide a distributed storage mechanism in a controlled environment and non-controllable environment.

#### - Virtual Computing Machine - Not yet
Pluggable virtual machine library

#### - Distributed Computing - Not yet
Pluggable edge computing library

#### - Privacy Data - Not yet
Optional privacy data model
 
#### - Privacy Computing - Not yet
Optional privacy data computing model


### Auxiliary library
- [Oolong Tea](https://github.com/placefortea/oolong_tea) - License Library, Node Admission Algorithm, PGP-based Trusted Uncentralized PKC Model.
- [Gossip Tea](https://github.com/placefortea/gossip_tea) - Gossip-based non-interactive consensus process library.
- [PBFT](https://github.com/placefortea/pbft_tea) - PBFT consensus.
- [KeyStore](https://github.com/placefortea/keystore_tea) - Private key and account information secure access library.
- [rcmerkle](https://github.com/rust-cc/rcmerkle) - Efficient Merkle tree calculation function and state machine calculation.
- [rckad](https://github.com/rust-cc/rckad) - S/Kademlia(DHT) implementation.
- [rcmixed](https://github.com/rust-cc/rcmixed) - A password hybrid system derived from PGP that has been used to encrypt private keys and accounts.


![TDN Image 3](./assets/TDN_3.jpg)

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Communication
- **TDN** - Telegram Group: https://t.me/placefortea
- **TDN中文** - QQ群: 757610599
