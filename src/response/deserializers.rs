use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    marker::PhantomData,
    fmt::{self, Display}
};
use serde::{
    Deserialize,
    de::{
        self,
        MapAccess,
        Visitor,
        SeqAccess,
        Deserializer,
        Unexpected,
    }
};
use serde_json::value::RawValue;
use lazy_regex::{regex_is_match, regex_captures};
use super::classinfo::ClassInfo;
use crate::types::{
    ClassInfoAppClass,
    ClassInfoAppMap
};

pub fn string_or_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + TryFrom<u64> + Deserialize<'de>,
    T::Err: Display,
{
    struct NumericVisitor<T> {
        marker: PhantomData<T>,
    }
    
    impl<T> NumericVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> de::Visitor<'de> for NumericVisitor<T>
    where 
        T: FromStr + TryFrom<u64> + Deserialize<'de>,
        T::Err: Display,
    {
        type Value = T;
    
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or a string")
        }
    
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match T::try_from(v) {
                Ok(c) => {
                    Ok(c)
                },
                Err(_e) => {
                    Err(de::Error::custom("Number too large to fit in target type"))
                }
            }
        }
    
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<T>().map_err(de::Error::custom)
        }
    }
    
    deserializer.deserialize_any(NumericVisitor::new())
}

pub fn from_int_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

// use serde::de::IntoDeserializer;
// pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
// where
//     T: FromStr,
//     T::Err: std::fmt::Display,
//     D: Deserializer<'de>
// {
//     let s = String::deserialize(deserializer)?;
    
//     T::from_str(&s).map_err(de::Error::custom)
// }

// pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
// where
//     D: serde::Deserializer<'de>,
//     T: serde::Deserialize<'de>,
// {
//     let opt = Option::<String>::deserialize(de)?;
//     let opt = opt.as_ref().map(String::as_str);
    
//     match opt {
//         None | Some("") => Ok(None),
//         Some(s) => T::deserialize(s.into_deserializer()).map(Some)
//     }
// }

pub fn from_fraudwarnings<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FraudWarningsVisitor;

    impl<'de> de::Visitor<'de> for FraudWarningsVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "" => Ok(None),
                other => Ok(Some(vec![other.to_string()])),
            }
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<String>()? {
                items.push(item);
            }

            Ok(Some(items))
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut items = Vec::new();
            
            while let Some((_key, v)) = access.next_entry::<String, String>()? {
                items.push(v);
            }
            
            Ok(Some(items))
        }
    }

    deserializer.deserialize_any(FraudWarningsVisitor)
}

pub fn into_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct DeserializeBoolVisitor;

    impl<'de> de::Visitor<'de> for DeserializeBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or a string")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                other => Err(de::Error::invalid_value(
                    Unexpected::Unsigned(other as u64),
                    &"zero or one",
                )),
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "0" => Ok(false),
                "1" => Ok(true),
                other => Err(de::Error::invalid_value(
                    Unexpected::Str(other),
                    &"zero or one",
                )),
            }
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    deserializer.deserialize_any(DeserializeBoolVisitor)
}

pub fn to_classinfo_map<'de, D>(deserializer: D) -> Result<HashMap<ClassInfoAppClass, Arc<ClassInfo>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoVisitor;

    impl<'de> Visitor<'de> for ClassInfoVisitor {
        type Value = HashMap<ClassInfoAppClass, Arc<ClassInfo>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of classinfos")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<HashMap<ClassInfoAppClass, Arc<ClassInfo>>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: HashMap<(u64, Option<u64>), Arc<ClassInfo>> = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(classinfo) = seq.next_element::<ClassInfo>()? {
                map.insert((classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
            }

            Ok(map)
        }
    }

    deserializer.deserialize_seq(ClassInfoVisitor)
}

pub fn hashmap_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct HashMapVisitor<T> {
        marker: PhantomData<Vec<T>>,
    }
    
    impl<T> HashMapVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }
    
    impl<'de, T> Visitor<'de> for HashMapVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;
        
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
        
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
    
            while let Some(v) = visitor.next_element::<T>()? {
                vec.push(v);
            }
    
            Ok(vec)
        }
        
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v {
                "" => Ok(Vec::new()),
                other => Err(de::Error::invalid_value(
                    Unexpected::Str(other),
                    &"zero or one",
                )),
            }
        }
        
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut items = Self::Value::new();
            
            while let Some((_key, v)) = access.next_entry::<String, T>()? {
                items.push(v);
            }
            
            Ok(items)
        }
    }
    
    deserializer.deserialize_any(HashMapVisitor::new())
}

pub fn deserialize_classinfo_map<'de, D>(deserializer: D) -> Result<ClassInfoAppMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoMapVisitor;
    
    impl<'de> Visitor<'de> for ClassInfoMapVisitor {
        type Value = ClassInfoAppMap;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
    
        fn visit_seq<M>(self, mut _seq: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            Ok(Self::Value::new())
        }
    
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            
            while let Some(key) = access.next_key::<String>()? {
                if regex_is_match!(r#"\d+"#, &key) {
                    let classinfo = access.next_value::<ClassInfo>()?;
                    
                    map.insert((classinfo.classid, classinfo.instanceid), Arc::new(classinfo));
                } else if let Ok(_invalid) = access.next_value::<u8>() {
                    // invalid key - discard
                }
            }
            
            Ok(map)
        }
    }
    
    deserializer.deserialize_any(ClassInfoMapVisitor)
}

pub fn deserialize_classinfo_map_raw<'de, D>(deserializer: D) -> Result<HashMap<ClassInfoAppClass, String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ClassInfoMapVisitor;
    
    impl<'de> Visitor<'de> for ClassInfoMapVisitor {
        type Value = HashMap<ClassInfoAppClass, String>;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }
    
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            
            while let Some(key) = access.next_key::<String>()? {
                if let Some((_, classid_string, instanceid_string)) = regex_captures!(r#"(\d+)_?(\d+)?"#, &key) {
                    let classid = classid_string.parse::<u64>().map_err(de::Error::custom)?;
                    let instanceid = match instanceid_string.parse::<u64>() {
                        Ok(instanceid) => Some(instanceid),
                        Err(_) => None,
                    };
                    let raw_value = access.next_value::<Box<RawValue>>()?;
                    let classinfo_string = raw_value.to_string();
                    
                    map.insert((classid, instanceid), classinfo_string);
                } else if let Ok(_invalid) = access.next_value::<u8>() {
                    // invalid key - discard
                }
            }
            
            Ok(map)
        }
    }
    
    deserializer.deserialize_any(ClassInfoMapVisitor)
}

pub fn option_str_to_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + TryFrom<u64> + Deserialize<'de>,
    T::Err: Display,
{
    struct OptionVisitor<T> {
        marker: PhantomData<Vec<T>>,
    }
    
    impl<T> OptionVisitor<T> {
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }
    }

    impl<'de, T> Visitor<'de> for OptionVisitor<T>
    where
        T: FromStr + TryFrom<u64> + Deserialize<'de>,
        T::Err: Display,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        
        fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match T::try_from(v) {
                Ok(c) => {
                    Ok(Some(c))
                },
                Err(_e) => {
                    Err(de::Error::custom("Number too large to fit in target type"))
                }
            }
        }
        
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.parse::<T>().map_err(de::Error::custom)?))
        }
    }

    deserializer.deserialize_any(OptionVisitor::new())
}