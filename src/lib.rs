// temp while in dev
#![allow(dead_code)]

cfg_if::cfg_if! {
    if #[cfg(feature = "kzg")] {
        pub use halo2_kzg as halo2;
    } else {
        // default feature
        pub use halo2_zcash as halo2;
    }
}

pub mod utils;
pub mod gadget;
pub mod circuit;
pub mod poseidon;
pub mod hash_to_field;
pub mod merkle;
pub mod client;
pub mod proof;
pub mod keys;

// #[cfg(target_arch = "wasm32")]
// pub mod build;