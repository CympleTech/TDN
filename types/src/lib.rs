//! Helper, When building the application, it is impossible to determine the data structure of the layers application, you can use this trait to constrain

pub mod group;
pub mod message;
pub mod primitives;
pub mod rpc;
pub mod storage;

// 1. single
// only one group, and only group's message. no layer.

// 2. std
// only one group, but multiple layers. (ESSE).
// one group is self's devices. (date storage and application control).
// multiple layers include chat with others, use other services.
// other services maybe distributed or centralized.
// if is centralized, it only upper to it, and it manager all it's lowers user.
// if is distributed, it need upper to its (groups), and it manager all it's lowers.

// 3. multiple
// multiple groups, multiple layers. (some domain service).

#[cfg(not(feature = "single"))]
pub mod layer;
