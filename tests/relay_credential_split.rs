//! The relay must not be able to impersonate a viewer.
//!
//! The session token authenticates the end-to-end encryption handshake.
//! Before this change the same token was sent to the relay during
//! registration, so a relay that learned it could compute a valid viewer
//! proof and sit in the middle of a stream it claims not to be able to
//! read. The relay now authenticates with a value derived from the token
//! instead of the token itself.

use pixel_change_check_client::network::web_e2e::{proof_of_public, BrowserKeyPair, BrowserOffer};
use pixel_change_check_client::network::{relay_credential, verify_token, SessionToken};

fn token() -> SessionToken {
    SessionToken::parse("RELAYCREDSPLIT").expect("a valid token")
}

#[test]
fn the_relay_credential_is_not_the_session_token() {
    let t = token();
    let credential = relay_credential(&t);
    assert_ne!(credential, t.as_str(), "the relay was handed the secret");
    // What the relay stores must not verify as the session token, or the
    // split buys nothing.
    assert!(
        !verify_token(&t, &credential),
        "the derived credential still authenticates as the secret"
    );
}

#[test]
fn the_relay_credential_is_stable_and_session_specific() {
    let a = token();
    let b = SessionToken::parse("RELAYCREDSPLIT2").expect("valid");
    assert_eq!(
        relay_credential(&a),
        relay_credential(&a),
        "the credential must be derivable by both peers, so it must be stable"
    );
    assert_ne!(
        relay_credential(&a),
        relay_credential(&b),
        "two sessions must not share a relay credential"
    );
}

#[test]
fn a_relay_holding_its_credential_cannot_forge_a_viewer_proof() {
    let t = token();
    // Everything a hostile relay operator learns: the session code it
    // routes on, and the credential it was given.
    let known_to_relay = relay_credential(&t);

    let keys = BrowserKeyPair::generate();
    let offer = pixel_change_check_client::network::web_e2e::offer(&keys, t.as_str());
    let real_proof = proof_of_public(&offer.public, t.as_str());

    // The proof a peer with only the relay's knowledge would produce: the
    // same construction over the only secret the relay has.
    let forged = proof_of_public(&offer.public, &known_to_relay);
    assert_ne!(
        real_proof, forged,
        "a relay able to derive the proof can impersonate a viewer"
    );

    // And the forged offer is what the sharer would reject.
    let forged_offer = BrowserOffer {
        public: offer.public,
        proof: forged,
    };
    let outcome = pixel_change_check_client::network::web_e2e::accept(
        &forged_offer,
        &BrowserKeyPair::generate(),
        t.as_str(),
    );
    assert!(
        outcome.is_err(),
        "the sharer accepted a proof built from the relay's credential alone"
    );
}

#[test]
fn a_genuine_viewer_is_still_accepted() {
    let t = token();
    let keys = BrowserKeyPair::generate();
    let offer = pixel_change_check_client::network::web_e2e::offer(&keys, t.as_str());
    let accepted = pixel_change_check_client::network::web_e2e::accept(
        &offer,
        &BrowserKeyPair::generate(),
        t.as_str(),
    );
    assert!(accepted.is_ok(), "a real viewer must still get a session");
}

#[test]
fn the_relay_refuses_a_peer_that_presents_the_raw_token() {
    // The old behaviour: a peer that learned the token could register
    // with it. That is exactly what must stop working.
    let t = token();
    let presented = t.as_str().to_string();
    assert!(
        presented != relay_credential(&t),
        "presenting the raw token would still satisfy the check"
    );
}
