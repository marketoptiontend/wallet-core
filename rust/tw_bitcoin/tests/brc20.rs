mod common;

use crate::common::{btc_info, dust_threshold, input, output, plan, sign, DUST};
use tw_bitcoin::entry::BitcoinEntry;
use tw_coin_entry::coin_entry::CoinEntry;
use tw_coin_entry::modules::plan_builder::PlanBuilder;
use tw_coin_entry::test_utils::test_context::TestCoinContext;
use tw_encoding::hex::{DecodeHex, ToHex};
use tw_proto::BitcoinV3::Proto;
use tw_proto::Common::Proto::SigningError;
use tw_utxo::sighash::SighashBase;

#[test]
fn coin_entry_sign_brc20_commit_reveal_transfer() {
    let coin = TestCoinContext::default();

    let alice_private_key = "e253373989199da27c48680e3a3fc0f648d50f9a727ef17a7fe6a4dc3b159129"
        .decode_hex()
        .unwrap();
    let alice_pubkey = "030f209b6ada5edb42c77fd2bc64ad650ae38314c8f451f3e36d80bc8e26f132cb"
        .decode_hex()
        .unwrap();

    let txid = "8ec895b4d30adb01e38471ca1019bfc8c3e5fbd1f28d9e7b5653260d89989008";
    let tx1 = Proto::Input {
        out_point: input::out_point(txid, 1),
        value: 26_400,
        sighash_type: SighashBase::All as u32,
        claiming_script: input::p2wpkh(alice_pubkey.clone()),
        ..Default::default()
    };

    let out1 = Proto::Output {
        value: 7_000,
        to_recipient: output::brc20_inscribe(alice_pubkey.clone(), "oadf", "20"),
    };

    // Change/return transaction.
    let out2 = Proto::Output {
        value: 16_400,
        to_recipient: output::p2wpkh(alice_pubkey.clone()),
    };

    let signing = Proto::SigningInput {
        private_keys: vec![alice_private_key.as_slice().into()],
        inputs: vec![tx1],
        outputs: vec![out1, out2],
        input_selector: Proto::InputSelector::UseAll,
        chain_info: btc_info(),
        dust_policy: dust_threshold(DUST),
        ..Default::default()
    };

    let signed = BitcoinEntry.sign(&coin, signing);
    assert_eq!(signed.error, SigningError::OK, "{}", signed.error_message);
    assert_eq!(
        signed.txid.to_hex(),
        "797d17d47ae66e598341f9dfdea020b04d4017dcf9cc33f0e51f7a6082171fb1"
    );

    let encoded = signed.encoded.to_hex();
    let transaction = signed.transaction.unwrap();

    assert_eq!(transaction.inputs.len(), 1);
    assert_eq!(transaction.outputs.len(), 2);
    assert_eq!(encoded, "02000000000101089098890d2653567b9e8df2d1fbe5c3c8bf1910ca7184e301db0ad3b495c88e0100000000ffffffff02581b000000000000225120e8b706a97732e705e22ae7710703e7f589ed13c636324461afa443016134cc051040000000000000160014e311b8d6ddff856ce8e9a4e03bc6d4fe5050a83d02483045022100a44aa28446a9a886b378a4a65e32ad9a3108870bd725dc6105160bed4f317097022069e9de36422e4ce2e42b39884aa5f626f8f94194d1013007d5a1ea9220a06dce0121030f209b6ada5edb42c77fd2bc64ad650ae38314c8f451f3e36d80bc8e26f132cb00000000");

    // https://www.blockchain.com/explorer/transactions/btc/797d17d47ae66e598341f9dfdea020b04d4017dcf9cc33f0e51f7a6082171fb1
    let txid = "797d17d47ae66e598341f9dfdea020b04d4017dcf9cc33f0e51f7a6082171fb1";
    let tx1 = Proto::Input {
        out_point: input::out_point(txid, 0),
        value: 7_000,
        sighash_type: SighashBase::All as u32,
        claiming_script: input::brc20_inscribe(alice_pubkey.clone(), "oadf", "20"),
        ..Default::default()
    };

    let out1 = Proto::Output {
        value: DUST,
        to_recipient: output::p2wpkh(alice_pubkey.clone()),
    };

    let signing = Proto::SigningInput {
        private_keys: vec![alice_private_key.as_slice().into()],
        inputs: vec![tx1],
        outputs: vec![out1],
        input_selector: Proto::InputSelector::UseAll,
        chain_info: btc_info(),
        // We enable deterministic Schnorr signatures here
        dangerous_use_fixed_schnorr_rng: true,
        dust_policy: dust_threshold(DUST),
        ..Default::default()
    };

    let plan = BitcoinEntry.plan_builder().unwrap().plan(&coin, &signing);
    plan::verify(
        &plan,
        plan::Expected {
            inputs: vec![7_000],
            outputs: vec![DUST],
            vsize_estimate: 132,
            fee_estimate: 7_000 - DUST,
            change: 0,
        },
    );

    // https://www.blockchain.com/explorer/transactions/btc/7046dc2689a27e143ea2ad1039710885147e9485ab6453fa7e87464aa7dd3eca
    let signed = BitcoinEntry.sign(&coin, signing.clone());
    sign::verify(&signing, &signed, sign::Expected {
        encoded: "02000000000101b11f1782607a1fe5f033ccf9dc17404db020a0dedff94183596ee67ad4177d790000000000ffffffff012202000000000000160014e311b8d6ddff856ce8e9a4e03bc6d4fe5050a83d03406a35548b8fa4620028e021a944c1d3dc6e947243a7bfc901bf63fefae0d2460efa149a6440cab51966aa4f09faef2d1e5efcba23ab4ca6e669da598022dbcfe35b0063036f7264010118746578742f706c61696e3b636861727365743d7574662d3800377b2270223a226272632d3230222c226f70223a227472616e73666572222c227469636b223a226f616466222c22616d74223a223230227d6821c00f209b6ada5edb42c77fd2bc64ad650ae38314c8f451f3e36d80bc8e26f132cb00000000",
        txid: "7046dc2689a27e143ea2ad1039710885147e9485ab6453fa7e87464aa7dd3eca",
        inputs: vec![7_000],
        outputs: vec![DUST],
        // `vsize` is different from the estimated value due to the signatures der serialization.
        vsize: 131,
        fee: 7_000 - DUST,
    });
}
