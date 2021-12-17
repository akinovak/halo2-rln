use halo2::{
    pasta::Fp
};

use num_bigint::BigUint;
use sha2::{Digest, Sha256, digest::FixedOutput};
use ::ff::{PrimeField};
use byte_io::*;
use std::convert::TryInto;

const PREFIX_RLN_HASH_TO_FIELD: &[u8; 17] = b"rln_hash_to_field";
const PREFIX_RLN_HASH_TO_FIELD_LO: &[u8; 20] = b"rln_hash_to_field_lo";
const PREFIX_RLN_HASH_TO_FIELD_HI: &[u8; 20] = b"rln_hash_to_field_hi";

pub fn hash_to_field(data: &[u8]) -> Fp {
    let mut hasher = Sha256::new();
    hasher.update(PREFIX_RLN_HASH_TO_FIELD);
    hasher.update(data);

    let mut hasher_to_lo = hasher.clone();
    let mut hasher_to_hi = hasher.clone();

    hasher_to_lo.update(PREFIX_RLN_HASH_TO_FIELD_LO);
    let result_1: [u8; 32] = hasher_to_lo.finalize_fixed().as_slice().try_into().unwrap();

    hasher_to_hi.update(PREFIX_RLN_HASH_TO_FIELD_HI);
    let result_2: [u8; 32] = hasher_to_hi.finalize_fixed().as_slice().try_into().unwrap();

    let lo = &BigUint::from_bytes_le(&result_1[..]);
    let hi = &BigUint::from_bytes_le(&result_2[..]);

    let combined: BigUint = lo + hi * (BigUint::from(1usize) << 256);
    let modulus: BigUint = BigUint::parse_bytes(b"40000000000000000000000000000000224698fc094cf91b992d30ed00000001", 16).unwrap();

    let field_element = combined % modulus;

    let mut buf = <Fp as PrimeField>::Repr::default();
    write_le(&field_element.to_bytes_le(), &mut buf);

    let fp = Fp::from_repr(buf).unwrap();
    fp
}

#[cfg(test)]
mod test {

    use super::hash_to_field;
    #[test]
    fn to_poin() {
        let s = format!("try to field");
        hash_to_field(s.as_bytes());
    }
}