#[cfg(test)]
mod tests {
    use super::super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::testutils::Events as _;
    use soroban_sdk::{Env, Symbol, TryFromVal, TryIntoVal};

    #[test]
    fn test_bid_refunded_event_emitted_when_outbid() {
        let env = Env::default();
        env.mock_all_auths();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let contract_id = env.register(Auction, ());
        let client = AuctionClient::new(&env, &contract_id);

        // Alice places initial bid
        client.place_bid(&Symbol::new(&env, "auc1"), &alice, &100_i128);

        // Bob outbids Alice
        client.place_bid(&Symbol::new(&env, "auc1"), &bob, &200_i128);

        // Capture events and assert BID_RFDN event present with correct prev_bidder and amount
        let events = env.events().all();
        assert!(!events.is_empty());
        // Find the last BID_RFDN event
        let mut found = false;
        for (_contract, topics, data) in events.iter().rev() {
            let t0: Symbol = Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
            if t0 == Symbol::new(&env, "BID_RFDN") {
                let event_data: events::BidRefundedEvent = data.try_into_val(&env).unwrap();
                assert_eq!(event_data.prev_bidder, alice);
                assert_eq!(event_data.amount, 100_i128);
                found = true;
                break;
            }
        }
        assert!(found, "BID_RFDN event not found");
    }
}
