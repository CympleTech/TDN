/// This is the interface of the group in the entire network,
/// you can customize the implementation of these methods,
/// you can set the permission or size of the group
pub trait Group {
    type JoinType;
    type JoinResultType;
    //type UpperType;
    //type LowerType;
}
