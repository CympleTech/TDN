[English](./README.md) / [中文](./README_zh.md)

- Start Date: 2019-06-03
- RFC PR #1
- Rust Issue #1

Summary 摘要
=======

设计一种可信的分布式网络,让每个人能够自由并且安全地管理自己的数据,开发者可以基于这套网络实现可交互,可计算,无服务器的应用,同时帮助应用间打通沟通和数据壁垒.

Motivation 动机
==========

Not yet

Detailed Design 详细设计
===============

### Architecture Design 架构设计
- 支持不同的数据结构
- 支持不同的共识算法
- 支持不同的许可机制
- 支持不同的帐号体系
- 应用间能够相互沟通
- 信任可以传递和积累

#### P2P & RPC Network 网络模块和基础设计
**Teatree** - [Code](https://github.com/placefortea/teatree) - [Design](https://github.com/placefortea/teatree/issues/1)

#### Distributed Strorage 分布式存储
**Black Tea** - [Code](https://github.com/placefortea/black_tea) - [Design](https://github.com/placefortea/black_tea/issues/1)

#### Virtual Computing Machine 虚拟机
Not yet

#### Distributed Computing 分布式计算
Not yet

#### Privacy Data & Privacy Computing 隐私数据和隐私计算
Not yet

#### Auxiliary library 辅助库
- [Oolong Tea](https://github.com/placefortea/oolong_tea) - 许可库, 节点准入算法.
- [PBFT](https://github.com/placefortea/pbft_tea) - PBFT 共识.
- [KeyStore](https://github.com/placefortea/keystore_tea) - 私钥和账户信息安全存取库.
- [GitHub On TDN](https://github.com/placefortea/git_tea) - 基于TDN的代码托管和共享仓库.

Drawbacks 缺点
=========

Not yet

Alternatives 备选方案
============

Not yet

Unresolved questions 尚未解决的问题
====================

Not yet
