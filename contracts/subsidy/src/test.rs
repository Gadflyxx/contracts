use super::*;
use soroban_sdk::testutils::Address as _;
#[test]
fn reimburses_once_and_requires_admin() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let vault = e.register(Subsidy, (&admin,));
    let recipient = Address::generate(&e);
    let asset = e.register_stellar_asset_contract_v2(admin).address();
    e.mock_all_auths();
    token::StellarAssetClient::new(&e, &asset).mint(&vault, &100);
    let client = SubsidyClient::new(&e, &vault);
    let id = BytesN::from_array(&e, &[1; 32]);
    client.reimburse(&id, &asset, &recipient, &10);
    assert_eq!(token::Client::new(&e, &asset).balance(&recipient), 10);
    assert!(client.try_reimburse(&id, &asset, &recipient, &10).is_err());
    e.set_auths(&[]);
    assert!(client
        .try_reimburse(&BytesN::from_array(&e, &[2; 32]), &asset, &recipient, &10)
        .is_err());
}
