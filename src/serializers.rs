
pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{de, Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer
    {
        serializer.collect_str(value)
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

pub mod option_string {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer
    {
        match value {
            Some(string) => serializer.collect_str(string),
            None => serializer.serialize_none()
        }
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>
    {
        let s: Option<String> = Option::<String>::deserialize(deserializer)?;
        
        if let Some(v) = s {
            return Ok(Some(v.parse::<T>().map_err(serde::de::Error::custom)?))
        }
            
        Ok(None)
    }
}

pub mod option_string_0_as_none {
    use std::fmt::Display;
    use std::str::FromStr;
    use serde::{Serializer, Deserialize, Deserializer};
    
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer
    {
        match value {
            Some(string) => serializer.collect_str(string),
            None => serializer.serialize_none()
        }
    }
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>
    {
        let s: Option<String> = Option::<String>::deserialize(deserializer)?;
        
        if let Some(v) = s {
            return Ok(match v.as_str() {
                "0" => None,
                v => Some(v.parse::<T>().map_err(serde::de::Error::custom)?)
            });
        }
            
        Ok(None)
    }
}

// pub mod option_color {
//     use crate::response::classinfo::Color;
//     use serde::{de, Serializer, Deserialize, Deserializer};
//     use crate::hex;
    
//     pub fn serialize<S>(value: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         match value {
//             Some(color) => serializer.collect_str(&hex::decimal_to_hex(*color)),
//             None => serializer.serialize_none()
//         }
//     }
    
//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
//     where
//         D: Deserializer<'de>
//     {
//         let s: Option<String> = Option::<String>::deserialize(deserializer)?;
        
//         if let Some(v) = s {
//             return match hex::parse_hex(&v) {
//                 Ok(v) => Ok(Some(v)),
//                 Err(err) => Err(de::Error::custom(err.to_string())),
//             };
//         }
            
//         Ok(None)
//     }
// }