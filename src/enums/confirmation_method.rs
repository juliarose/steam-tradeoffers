use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde_repr::{Serialize_repr, Deserialize_repr};
use strum_macros::{Display, EnumString};

#[derive(Serialize_repr, Deserialize_repr, Display, EnumString, Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum ConfirmationMethod {
    None = 0,
    Email = 1,
    MobileApp = 2,
}