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

#[test]
fn failed_transfer_rolls_back_reimbursement_and_can_be_retried() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let vault = e.register(Subsidy, (&admin,));
    let asset = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let recipient = Address::generate(&e);
    let client = SubsidyClient::new(&e, &vault);
    let id = BytesN::from_array(&e, &[3; 32]);
    e.mock_all_auths();
    assert_eq!(client.admin(), admin);
    assert_eq!(
        client.try_reimburse(&id, &asset, &recipient, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert!(client.try_reimburse(&id, &asset, &recipient, &10).is_err());
    assert!(!client.is_reimbursed(&id));
    token::StellarAssetClient::new(&e, &asset).mint(&vault, &10);
    client.reimburse(&id, &asset, &recipient, &10);
    assert!(client.is_reimbursed(&id));
    assert_eq!(token::Client::new(&e, &asset).balance(&recipient), 10);
    assert_eq!(
        client.try_reimburse(&id, &asset, &recipient, &10),
        Err(Ok(Error::AlreadyReimbursed))
    );
}
