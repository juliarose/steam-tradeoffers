use super::confirmation::{Confirmation, ConfirmationType};
use crate::{
    SteamID,
    time,
    error::ParseHtmlError,
};
use hmacsha1::hmac_sha1;
use sha1::{Sha1, Digest};
use lazy_regex::regex_replace_all;
use scraper::{Html, Selector, element_ref::ElementRef};

pub fn build_time_bytes(time: i64) -> [u8; 8] {
    time.to_be_bytes()
}
    
pub fn generate_confirmation_hash_for_time(time: i64, tag: &str, identity_secret: &String) -> String {
    let decode: &[u8] = &base64::decode(&identity_secret).unwrap();
    let time_bytes = build_time_bytes(time);
    let tag_bytes = tag.as_bytes();
    let array = [&time_bytes, tag_bytes].concat();
    let hash = hmac_sha1(decode, &array);
    
    base64::encode(hash)
}

pub fn get_device_id(steamid: &SteamID) -> String {
    let mut hasher = Sha1::new();

    hasher.update(u64::from(*steamid).to_string().as_bytes());
    
    let result = hasher.finalize();
    let hash = result.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let device_id = regex_replace_all!(
        r#"^([0-9a-f]{8})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{4})([0-9a-f]{12}).*$"#i,
        &hash,
        |_, a, b, c, d, e| format!("{}-{}-{}-{}-{}", a, b, c, d, e),
    );
    
    format!("android:{}", device_id)
}

pub fn parse_confirmations(text: String) -> Result<Vec<Confirmation>, ParseHtmlError> {
    fn parse_description(element: ElementRef, description_selector: &Selector) -> Result<Confirmation, ParseHtmlError> {
        let description: Option<_> = element.select(description_selector).next();
        let data_type = element.value().attr("data-type");
        let id = element.value().attr("data-confid");
        let key = element.value().attr("data-key");
        let creator = element.value().attr("data-creator");
        
        // check contents before unwrapping
        if description.is_none() || data_type.is_none() || key.is_none() || creator.is_none() {
            return Err(ParseHtmlError::Malformed("Unexpected description format"));
        }
        
        let description = description
            .unwrap()
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let conf_type = data_type
            .unwrap()
            .try_into()
            .unwrap_or(ConfirmationType::Unknown);
        
        Ok(Confirmation {
            id: id.unwrap().parse::<u64>()?,
            key: key.unwrap().parse::<u64>()?,
            conf_type,
            description,
            creator: creator.unwrap().parse::<u64>()?,
        })
    }

    let fragment = Html::parse_fragment(&text);
    // these should probably never fail
    let mobileconf_empty_selector = Selector::parse("#mobileconf_empty").unwrap();
    let mobileconf_done_selector = Selector::parse(".mobileconf_done").unwrap();
    let div_selector = Selector::parse("div").unwrap();
    
    if let Some(element) = fragment.select(&mobileconf_empty_selector).next() {
        if mobileconf_done_selector.matches(&element) {
            if let Some(element) = element.select(&div_selector).nth(1) {
                let error_message = element
                    .text()
                    .collect::<String>();
                
                return Err(ParseHtmlError::Response(error_message));
            } else {
                return Ok(Vec::new());
            }
        } else {
            return Ok(Vec::new());
        }
    }
    
    let confirmation_list_selector = Selector::parse(".mobileconf_list_entry").unwrap();
    let description_selector = Selector::parse(".mobileconf_list_entry_description").unwrap();
    let confirmations = fragment.select(&confirmation_list_selector)
        .map(|description| parse_description(description, &description_selector))
        .collect::<Result<Vec<Confirmation>, ParseHtmlError>>()?;
    
    Ok(confirmations)
}

pub fn server_time(time_offset: i64) -> i64 {
    time::get_system_time() as i64 + time_offset
}