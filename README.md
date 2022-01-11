[![crate](https://img.shields.io/badge/crates.io-v0.6.2-green.svg)](https://crates.io/crates/tdn) [![doc](https://img.shields.io/badge/docs.rs-v0.6.2-blue.svg)](https://docs.rs/tdn)

<h1 align="center"><img src="https://cympletech.com/logo/tdn_words.png" alt="TDN"></h1>

*TDN: Let data transfer safely, make the network stable, secure, and highly available.*

As the network interconnection, TDN includes peer-to-peer, centralized gateway, proxy & relay service, LAN, RPC, Near, etc.

As the framework of decentralized applications, TDN uses the `Layer` and `Group` models. We built this framework because we feel that the blockchain is very limited. If you want a more open and free distributed application development technology, and Pluggable, lightweight application framework, TDN can satisfy you.

## Example
- install `lastest` Rust and `cd ./tdn`
- `cargo run --example simple` Congratulation, you are running a trusted distributed network :)

[more sample](./tdn/examples)

## Features
- Support different data structures
- Support different consensus algorithms
- Support different permission mechanisms
- Support different account systems
- Applications can communicate with others
- Trust can be passed on and accumulated

## 4-type Applications
TDN can build 4-type network for applications.
Use `feature` to control. (Tip: one group, is one chain, is one application)

- **single**. Support only one group, and no layers. Like `Bitcoin`.
- **std** (default). Support only one group, and multiple layers (cross-chain, Interactive with other groups).
- **multiple**. Support many groups (in one runtime, support many applications), and no layers. Like `Ethereum`.
- **full**. Support many groups, and multiple layers.

## Architecture
![TDN with future](https://cympletech.com/images/TDN_future.jpg)

![TDN Groups And Layers](https://cympletech.com/images/TDN_groups_layers.jpg)

## Pluggable library
- [tdn-did](./did) decentralized identity.
- [tdn-storage](./storage) Storage library, include Local file, Local db, Distributed db, Decentralized db.
- [tdn-permission](https://github.com/cympletech/tdn-permission) Group permission library, include permissioned & permissionless group algorithm.

## Projects used it
- [ESSE](https://github.com/CympleTech/esse) - **full-type** - An open source encrypted peer-to-peer system. Own data, own privacy.
- [group-chat](https://github.com/CympleTech/group-chat) - **multiple-type** - Group Chat for ESSE and TDN Ecological.

## For more information, please visit:
- Website: https://cympletech.com
- Twitter: https://twitter.com/cympletech
- Discord: https://discord.gg/UfFjp6Kaj4
- E-mail: contact@cympletech.com

### Core QA
1. **What is Layers & Groups?**
Simply `Group` is application, one application is one group, `Layer` is communication between apps. In the Group, users can define everything. The Layer is divided into upper and lower, upper is this application depends on others, lower is others depends on this. If app is an independent application that does not interact with other applications, then you can completely ignore Layer.

2. **Different consensus?**
Consensus is expensive. Not all applications require global consensus to be used. Local consensus can speed up the usability of applications. Local consensus can be different. Users can define self-consensus in different applications. The results of the consensus can also be sent to different applications through the layer. In this way, the consensus results can be passed to the upper layer. After the upper layer consensus is obtained, a larger range and stronger consensus irreversible result can be formed.

3. **Different permission?**
Why do we need a different permission mechanism? Because in different applications, some applications require an open permissionless environment, such as Bitcoin, and some require a permissioned environment. For example, We built a synchronization which is distributed between my own devices is inaccessible to outsiders. At the same time, `Layer` supports applications in different permission environments, allowing data interaction.

4. **Different block & transaction data structure?**
Similarly, different applications have completely different data structures. Therefore, in the TDN, we use a common byte stream format. Users can customize the data format of the application. The communication with the TDN is only serialization and deserialization.

5. **Different account system?**
In the TDN, there will be a default p2p network address (PeerId) in the p2p system. The user can use the PeerId to replace the ip address. The TDN will search the PeerId in the network and establish a connection. At the same time, the user can also Fully customize the account model and connection verification method of the application.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
