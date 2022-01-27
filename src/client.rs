use crate::halo2::pasta::Fp;
use crate::poseidon::{Hash, ConstantLength, P128Pow5T3};
use crate::hash_to_field::hash_to_field;
use ff::*; 

pub fn calculate_output(secret: Fp, epoch: Fp, signal: &str) -> (Fp, Fp, Fp) {
    let signal = hash_to_field(signal.as_bytes());
    let coef = Hash::init(P128Pow5T3, ConstantLength::<2>).hash([secret, epoch]);
    let y = coef * signal + secret;
    let nullifier = Hash::init(P128Pow5T3, ConstantLength::<1>).hash([coef]);

    (y, nullifier, signal)
}

pub fn retrieve_secret(x1: Fp, y1: Fp, x2: Fp, y2:Fp) -> Fp {
    let slope = (y2 - y1) * (x2 - x1).invert().unwrap();
    y1 - slope * x1
}