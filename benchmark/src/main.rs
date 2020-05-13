use anyhow::Result;
use iota::bundle::TransactionField;
use iota::crypto::Kerl;
use iota::signing::{IotaSeed, Seed};
use iota::ternary::{T1B1Buf, TryteBuf};
use iota_conversion::Trinary;

async fn get_new_address() -> Result<()> {
    // Create seed from your seed trytes
    let seed = IotaSeed::<Kerl>::from_buf(
        TryteBuf::try_from_str(
            "RVORZ9SIIP9RCYMREUIXXVPQIPHVCNPQ9HZWYKFWYWZRE9JQKG9REPKIASHUUECPSQO9JT9XNMVKWYGVA",
        )
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>(),
    )
    .unwrap();

    // The response of get_new_address is a tuple of an adress with its corresponding index from seed.
    let (_, address) = iota::Client::new("https://nodes.comnet.thetangle.org")?
        .get_new_address()
        .seed(&seed)
        .security(3)
        .generate()
        .await
        .unwrap();

    println!(
        "{:?}",
        address.to_inner().as_i8_slice().trytes()
    );
    Ok(())
}

#[smol_potat::main]
async fn main() -> Result<()> {
    for _ in 1..10 {
        get_new_address().await?;
    }

    Ok(())
}
