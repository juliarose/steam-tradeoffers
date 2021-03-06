use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

#[derive(Serialize_repr, Deserialize_repr, Display, EnumString, Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum TradeStatus {
    Init = 0,
    PreCommitted = 1,
    Committed = 2,
    Complete = 3,
    Failed = 4,
    PartialSupportRollback = 5,
    FullSupportRollback = 6,
    SupportRollbackSelective = 7,
    RollbackFailed = 8,
    RollbackAbandoned = 9,
    InEscrow = 10,
    EscrowRollback = 11,
}