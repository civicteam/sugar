use crate::common::*;
use std::{thread, time::Duration};

pub async fn get_bundlr_solana_address(http_client: &HttpClient) -> Result<String> {
    let data = http_client
        .get("https://node1.bundlr.network/info")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let addresses = data.get("addresses").unwrap();

    let solana_address = addresses
        .get("solana")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    Ok(solana_address)
}

pub async fn fund_bundlr_address(
    program: &Program,
    http_client: &HttpClient,
    bundlr_address: Pubkey,
    payer: &Keypair,
    amount: u64,
) -> Result<Response> {
    let ix = system_instruction::transfer(&payer.pubkey(), &bundlr_address, amount);

    let recent_blockhash = program.rpc().get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    thread::sleep(Duration::from_millis(1000));

    println!("Funding bundlr with {amount} lamports");
    let sig = program
        .rpc()
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::confirmed(),
        )?;

    let mut map = HashMap::new();
    map.insert("tx_id", sig.to_string());

    let response = http_client
        .post("https://node1.bundlr.network/account/balance/solana")
        .json(&map)
        .send()
        .await?;

    Ok(response)
}

pub async fn get_bundlr_balance(http_client: &HttpClient, address: &str) -> Result<u64> {
    let response = http_client
        .get(format!(
            "https://node1.bundlr.network/account/balance/solana/?address={address}"
        ))
        .send()
        .await?
        .json::<Value>()
        .await?;

    println!("response: {response:?}");
    let value = response.get("balance").unwrap();
    println!("value: {value:?}");

    // Bundlr API returns balance as a number if it's zero but as a string if it's not. :-(
    let balance = if value.is_number() {
        value.as_u64().unwrap()
    } else {
        value.as_str().unwrap().parse::<u64>().unwrap()
    };

    println!("balance: {balance:?}");

    Ok(balance)
}

pub async fn get_bundlr_fee(http_client: &HttpClient, data_size: u64) -> Result<u64> {
    let required_amount = http_client
        .get(format!(
            "https://node1.bundlr.network/price/solana/{data_size}"
        ))
        .send()
        .await?
        .text()
        .await?
        .parse::<u64>()?;

    Ok(required_amount)
}
