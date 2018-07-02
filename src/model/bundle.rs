use super::transaction::Transaction;
use crate::pow::Kerl;
use crate::pow::{Sponge, HASH_LENGTH};
use crate::utils::{converter, trit_adder};
use serde_json;
use std::fmt;

const EMPTY_HASH: &str =
    "999999999999999999999999999999999999999999999999999999999999999999999999999999999";

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    bundle: Vec<Transaction>,
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).unwrap_or_default()
        )
    }
}

impl Bundle {
    pub fn new(transactions: &[Transaction]) -> Bundle {
        Bundle {
            bundle: transactions.to_vec(),
        }
    }

    pub fn bundle(&self) -> &[Transaction] {
        &self.bundle
    }

    pub fn bundle_mut(&mut self) -> &mut [Transaction] {
        &mut self.bundle
    }

    pub fn add_entry(
        &mut self,
        signature_message_length: usize,
        address: &str,
        value: i64,
        tag: &str,
        timestamp: i64,
    ) {
        for i in 0..signature_message_length {
            let mut trx = Transaction::default();
            *trx.address_mut() = Some(address.to_string());
            *trx.timestamp_mut() = Some(timestamp);
            *trx.tag_mut() = Some(tag.to_string());
            match i {
                0 => *trx.value_mut() = Some(value),
                _ => *trx.value_mut() = Some(0),
            }
            self.bundle.push(trx);
        }
    }

    pub fn add_trytes(&mut self, signature_fragments: &[String]) {
        let empty_signature_fragment = "9".repeat(2187);
        let empty_hash = EMPTY_HASH;
        let empty_timestamp = "9".repeat(9);

        for (i, bundle) in self.bundle.iter_mut().enumerate() {
            *bundle.signature_fragments_mut() =
                if signature_fragments.len() <= 1 || signature_fragments[i].is_empty() {
                    Some(empty_signature_fragment.clone())
                } else {
                    Some(signature_fragments[i].clone())
                };
            *bundle.trunk_transaction_mut() = Some(empty_hash.to_string());
            *bundle.branch_transaction_mut() = Some(empty_hash.to_string());
            *bundle.attachment_timestamp_mut() = Some(empty_timestamp.clone());
            *bundle.attachment_timestamp_lower_bound_mut() = Some(empty_timestamp.clone());
            *bundle.attachment_timestamp_upper_bound_mut() = Some(empty_timestamp.clone());
            *bundle.nonce_mut() = Some("9".repeat(27));
        }
    }

    pub fn finalize(&mut self) {
        let mut valid_bundle = false;
        let mut kerl = Kerl::default();
        while !valid_bundle {
            kerl.reset();
            for bundle in &mut self.bundle {
                let value_trits = converter::trits_with_length(bundle.value().unwrap(), 81);
                let timestamp_trits = converter::trits_with_length(bundle.timestamp().unwrap(), 27);
                let current_index_trits =
                    converter::trits_with_length(bundle.current_index().unwrap(), 27);
                let last_index_trits =
                    converter::trits_with_length(bundle.last_index().unwrap(), 27);
                let bundle_essence = converter::trits_from_string(
                    &(bundle.address().unwrap_or_default().to_string()
                        + &converter::trytes(&value_trits)
                        + &bundle.obsolete_tag().unwrap_or_default()
                        + &converter::trytes(&timestamp_trits)
                        + &converter::trytes(&current_index_trits)
                        + &converter::trytes(&last_index_trits)),
                );
                kerl.absorb(&bundle_essence);
            }
            let mut hash = [0; HASH_LENGTH];
            kerl.squeeze(&mut hash);
            let hash_trytes = converter::trytes(&hash);
            for bundle in &mut self.bundle {
                *bundle.bundle_mut() = Some(hash_trytes.clone());
            }
            let normalized_hash = Bundle::normalized_bundle(&hash_trytes.clone());
            if normalized_hash.contains(&13) {
                let increased_tag = trit_adder::add(
                    &converter::trits_from_string(
                        &self.bundle[0].obsolete_tag().unwrap_or_default(),
                    ),
                    &[1],
                );
                *self.bundle[0].obsolete_tag_mut() = Some(converter::trytes(&increased_tag));
            } else {
                valid_bundle = true;
            }
        }
    }

    pub fn normalized_bundle(bundle_hash: &str) -> [i8; 81] {
        let mut normalized_bundle = [0; 81];
        for i in 0..3 {
            let mut sum = 0;
            for j in 0..27 {
                let mut t = String::new();
                t.push(bundle_hash.chars().nth(i * 27 + j).unwrap());
                normalized_bundle[i * 27 + j] = converter::value(&converter::trits_from_string(&t));
                sum += normalized_bundle[i * 27 + j];
            }
            if sum >= 0 {
                while sum > 0 {
                    for j in 0..27 {
                        if normalized_bundle[i * 27 + j] > -13 {
                            normalized_bundle[i * 27 + j] -= 1;
                            break;
                        }
                    }
                    sum -= 1;
                }
            } else {
                while sum < 0 {
                    for j in 0..27 {
                        if normalized_bundle[i * 27 + j] < 13 {
                            normalized_bundle[i * 27 + j] += 1;
                            break;
                        }
                    }
                    sum += 1;
                }
            }
        }
        normalized_bundle
    }
}
