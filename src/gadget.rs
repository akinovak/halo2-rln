pub mod merkle;
pub mod swap;
pub mod poseidon;
pub mod rln;
// use halo2::arithmetic::FieldExt;
// use crate::gadget::mux::{MuxChip};
// use crate::circuit::Config;

// impl<F: FieldExt> Config<F> {
//     pub(super) fn construct_mux_chip(&self) -> MuxChip<F> {
//         MuxChip::construct(self.mux_config.clone())
//     }
// }