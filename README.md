[English](./README.md) / [中文](./README_zh.md)

- Start Date: 2019-06-03
- RFC PR #: 1
- Rust Issue #: 1

Summary
=======

Design a trusted distributed network that allows everyone to manage their data freely and securely. Developers can implement interactive, computable, serverless applications based on this network, while helping to communicate and data between applications.

Motivation
==========

Not yet

Detailed Design
===============

### Architecture Design
- Support different data structures
- Support different consensus algorithms
- Support different permission mechanisms
- Support different account systems
- Applications can communicate with others
- Trust can be passed on and accumulated

### P2P & RPC Network
**Teatree** - [Code](https://github.com/placefortea/teatree) - [Design](https://github.com/placefortea/teatree/issues/1)

### Distributed Strorage
**Black Tea** - [Code](https://github.com/placefortea/black_tea) - [Design](https://github.com/placefortea/black_tea/issues/1)

### Virtual Computing Machine
Not yet

### Distributed Computing
Not yet

### Privacy Data & Privacy Computing
Not yet

#### Auxiliary library
- [Oolong Tea](https://github.com/placefortea/oolong_tea) - permissioned library for Distribution & Blockchain.
- [PBFT](https://github.com/placefortea/pbft_tea) - PBFT Consensus.
- [KeyStore](https://github.com/placefortea/keystore_tea) - Private Key & Account security store.
- [Git On TDN](https://github.com/placefortea/git_tea) - Github On Trusted Distributed Network/Github on Blockchain.

Drawbacks
=========

Not yet

Alternatives
============

Not yet

Unresolved questions
====================

Not yet
