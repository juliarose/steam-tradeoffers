mod confirmation;
mod helpers;

pub use confirmation::{Confirmation, ConfirmationType};

use serde::Deserialize;
use reqwest::cookie::Jar;
use url::{Url, ParseError};
use reqwest_middleware::ClientWithMiddleware;
use std::{collections::HashMap, sync::{Arc, RwLock}};
use crate::{
    SteamID,
    error::Error,
    helpers::{
        get_default_middleware,
        parses_response,
    },
};

const HOSTNAME: &str = "https://steamcommunity.com";
const USER_AGENT_STRING: &str = "Mozilla/5.0 (Linux; U; Android 4.1.1; en-us; Google Nexus 4 - 4.1.1 - API 16 - 768x1280 Build/JRO03S) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Mobile Safari/534.30";

#[derive(Debug)]
pub struct MobileAPI {
    client: ClientWithMiddleware,
    pub cookies: Arc<Jar>,
    pub language: String,
    pub steamid: SteamID,
    pub identity_secret: Option<String>,
    pub sessionid: Arc<RwLock<Option<String>>>,
}

impl MobileAPI {
    pub fn new(
        cookies: Arc<Jar>,
        steamid: SteamID,
        language: String,
        identity_secret: Option<String>,
    ) -> Self {
        let url = HOSTNAME.parse::<Url>().unwrap();
        let client = get_default_middleware(Arc::clone(&cookies), USER_AGENT_STRING);
        
        cookies.add_cookie_str("mobileClientVersion=0 (2.1.3)", &url);
        cookies.add_cookie_str("mobileClient=android", &url);
        cookies.add_cookie_str("Steam_Language=english", &url);
        cookies.add_cookie_str("dob=", &url);
        cookies.add_cookie_str(format!("steamid={}", u64::from(steamid)).as_str(), &url);
        
        Self {
            client,
            steamid,
            identity_secret,
            language,
            cookies,
            sessionid: Arc::new(RwLock::new(None)),
        }
    }
    
    fn get_uri(&self, pathname: &str) -> String {
        format!("{}{}", HOSTNAME, pathname)
    }
    
    // probably would never fail
    fn set_cookies(&self, cookies: &Vec<String>) -> Result<(), ParseError> {
        let url = HOSTNAME.parse::<Url>()?;
        
        for cookie_str in cookies {
            self.cookies.add_cookie_str(cookie_str, &url);
        }
        
        Ok(())
    }
    
    pub fn set_session(&self, sessionid: &str, cookies: &Vec<String>) -> Result<(), ParseError> {
        let mut sessionid_write = self.sessionid.write().unwrap();
        
        *sessionid_write = Some(sessionid.to_string());
        
        self.set_cookies(cookies)?;
        
        Ok(())
    }
    
    async fn get_confirmation_query_params<'a>(&self, tag: &str) -> Result<HashMap<&'a str, String>, Error> {
        if self.identity_secret.is_none() {
            return Err(Error::Parameter("No identity secret"));
        }
        
        // let time = self.get_server_time().await?;
        let time = helpers::server_time(0);
        let key = helpers::generate_confirmation_hash_for_time(
            time,
            tag,
            // safe - is_none checked above
            &self.identity_secret.clone().unwrap(),
        );
        let mut params: HashMap<&str, String> = HashMap::new();
        
        // self.device_id.clone()
        params.insert("p", helpers::get_device_id(&self.steamid));
        params.insert("a", u64::from(self.steamid).to_string());
        params.insert("k", key);
        params.insert("t", time.to_string());
        params.insert("m", "android".into());
        params.insert("tag", tag.into());
        
        Ok(params)
    }
    
    pub async fn send_confirmation_ajax(&self, confirmation: &Confirmation, operation: String) -> Result<(), Error>  {
        #[derive(Debug, Clone, Copy, Deserialize)]
        struct SendConfirmationResponse {
            pub success: bool,
        }
        
        let mut query = self.get_confirmation_query_params("conf").await?;
        
        query.insert("op", operation);
        query.insert("cid", confirmation.id.to_string());
        query.insert("ck", confirmation.key.to_string());

        let uri = self.get_uri("/mobileconf/ajaxop");
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        // let body: SendConfirmationResponse = parses_response(response).await?;
        let body: SendConfirmationResponse = parses_response(response).await?;
        
        if !body.success {
            return Err(Error::Response("Confirmation unsuccessful. The confirmation may no longer exist or another trade may be going through. Check confirmations again to verify.".into()));
        }
        
        Ok(())
    }

    pub async fn accept_confirmation(&self, confirmation: &Confirmation) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, "allow".into()).await
    }

    pub async fn deny_confirmation(&self, confirmation: &Confirmation) -> Result<(), Error> {
        self.send_confirmation_ajax(confirmation, "cancel".into()).await
    }
    
    pub async fn get_trade_confirmations(&self) -> Result<Vec<Confirmation>, Error> {
        let uri = self.get_uri("/mobileconf/conf");
        let query = self.get_confirmation_query_params("conf").await?;
        let response = self.client.get(&uri)
            .header("X-Requested-With", "com.valvesoftware.android.steam.community")
            .query(&query)
            .send()
            .await?;
        let body = response.text().await?;
        let confirmations = helpers::parse_confirmations(body)?;
        
        Ok(confirmations)
    }
}
