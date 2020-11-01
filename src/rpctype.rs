pub enum Tag {
    Invalid = -1,
    MetaTypeId = 1,
    MetaTypeNameSpaceId,
    USER = 8
}

pub enum NameSpaceID
{
    Global = 0,
    Elesys,
    Eyas,
}

#[allow(non_snake_case)]
pub mod GlobalNS
{
    pub enum MetaTypeID
    {
        ChainPackRpcMessage = 1,
        RpcConnectionParams,
        TunnelCtl,
        AccessGrantLogin,
        ValueChange,
    }
}
