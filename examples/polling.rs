use steam_tradeoffers::{
    TradeOfferManager,
    response::{TradeOffer, Asset},
    enums::TradeOfferState,
    error::Error,
    SteamID,
};

fn assets_item_names<'a>(
    assets: &'a Vec<Asset>,
) -> Vec<&'a str> {
    assets
        .iter()
        .map(|item| item.classinfo.market_hash_name.as_ref())
        .collect::<Vec<_>>()
}

async fn accept_offer(
    manager: &TradeOfferManager,
    offer: &TradeOffer,
) -> Result<(), Error> {
    let accepted_offer = manager.accept_offer(&offer).await?;
    
    if accepted_offer.needs_mobile_confirmation {
        manager.confirm_offer(&offer).await
    } else {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steamid = SteamID::from(0);
    let api_key = String::from("key");
    let manager = TradeOfferManager::builder(steamid.clone(), api_key)
        .identity_secret(String::from("secret"))
        .build();
    let sessionid = "sessionid";
    let cookies = vec![String::from("cookie=value")];
    
    manager.set_session(sessionid, &cookies)?;
    
    let items = manager.get_inventory(&steamid, 440, 2, true).await?;
    
    println!("{} items in your inventory", items.len());
    
    // gets changes to trade offers for account
    for (offer, old_state) in manager.do_poll(true).await? {
        if let Some(state) = old_state {
            println!(
                "Offer {} changed state: {} -> {}",
                offer,
                state,
                offer.trade_offer_state
            );
        } else if offer.trade_offer_state == TradeOfferState::Active && !offer.is_our_offer {
            println!(
                "New offer {}",
                offer
            );
            println!(
                "Receiving: {:?}",
                assets_item_names(&offer.items_to_receive)
            );
            println!(
                "Giving: {:?}",
                assets_item_names(&offer.items_to_give)
            );
            
            // free items
            if offer.items_to_give.is_empty() {
                if let Err(error) = accept_offer(&manager, &offer).await {
                    println!("Error accepting offer {}: {}", offer, error);
                } else {
                    println!("Accepted offer {}", offer);
                }
            }
        }
    }
    
    Ok(())
}