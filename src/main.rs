use bdk::{Wallet, SyncOptions, FeeRate, SignOptions};
use bdk::wallet::AddressIndex;
use bdk::database::MemoryDatabase;
use bdk::blockchain::{ElectrumBlockchain, Blockchain};
use bdk::electrum_client::Client;
use bdk::bitcoin::{self, Address};
use std::str::FromStr;

fn main() -> Result<(), bdk::Error> {
    //Initiate a wallet in testnet
    let wallet = Wallet::new(
        "tr(tprv8ZgxMBicQKsPf6WJ1Rr8Zmdsr6MaACS5K3tHw3QDQmFbkEsdnG3zAZzhjEgEtetL1jwZ5VAL85UaaFzUpAZPrS7aGkQ3GdM75xPu4sUxSiF/*)",
        None,
        bitcoin::Network::Testnet,
        MemoryDatabase::default(),
    )?;

    //generate an address
    let address = wallet.get_address(AddressIndex::New)?;
    println!("Thins is your wallet address: {}", address);

    //connect to Electrum server and save the blockchain
    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client);

    //sync wallet to the blockchain received
    wallet.sync(&blockchain, SyncOptions::default())?;

    //get the balance from your wallet
    let balance = wallet.get_balance()?;
    println!("This is your wallet balance: {}", balance);

    //build a transaction psbt
    let mut builder = wallet.build_tx();
    let recipient_address = Address::from_str("tb1qlj64u6fqutr0xue85kl55fx0gt4m4urun25p7q").unwrap();

    builder
        .drain_wallet()
        .drain_to(recipient_address.script_pubkey())
        .fee_rate(FeeRate::from_sat_per_vb(2.0))
        .enable_rbf();
    let (mut psbt, tx_details) = builder.finish()?;
    println!("This is our psbt: {}", psbt);
    println!("These are the details of the tx: {:?}", tx_details);

    //Sign the PSBT
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    println!("Is my PSBT Signed? {}", finalized);
    println!("This is my PSBT finalized: {}", psbt);

    
    let tx = psbt.extract_tx();
    let tx_id = tx.txid();
    println!("this is my Bitcoin tx: {}", bitcoin::consensus::encode::serialize_hex(&tx));
    println!("this is mny tx id: {}", tx_id);

    //Broadcast the transaction
    blockchain.broadcast(&tx)?;

    Ok(())
}