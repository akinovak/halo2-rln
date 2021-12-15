// use halo2::{
//     plonk::{Error},
//     circuit::{Layouter},
//     arithmetic::FieldExt,
// };
// mod chip;

// use std::marker::PhantomData;

// use crate::gadget::swap::{SwapInstruction};
// pub use chip::{MerkleConfig, MerkleChip};
// use crate::utils::{UtilitiesInstructions};


// pub trait MerkleInstructions<F:FieldExt>: UtilitiesInstructions<F> {
//     fn hash_layer(
//         &self, 
//         layouter: impl Layouter<F>,
//         left: Self::Var,
//         right: Self::Var,
//         level: usize
//     ) -> Result<Self::Var, Error>;
// }

// #[derive(Clone, Debug)]
// pub struct InclusionProof<
//     F: FieldExt,
//     const DEPTH: usize,
// >
// {
//     pub merkle_chip: MerkleChip<F>,
//     pub siblings: [Option<F>; DEPTH],
//     pub leaf_pos: [Option<bool>; DEPTH],
//     pub _marker: PhantomData<F>,
// }

// impl
// <
//     F: FieldExt,
//     const DEPTH: usize,
// >  InclusionProof<F, DEPTH>
// {
//     pub fn calculate_root(
//         &self, 
//         mut layouter: impl Layouter<F>,
//         leaf: <MerkleChip<F> as UtilitiesInstructions<F>>::Var,
//     ) -> Result<<MerkleChip<F> as UtilitiesInstructions<F>>::Var, Error> {

//         let mut node = leaf;

//         let chips = vec![self.merkle_chip.clone(); DEPTH];

//         for (level, ((sibling, pos), chip)) in self.siblings.iter().zip(self.leaf_pos.iter()).zip(chips).enumerate() {
//             let pair = {
//                 let pair = (node, *sibling);
//                 chip.swap(layouter.namespace(|| format!("swap pair on level {}", level)), pair, *pos)?
//             };

//             node = chip.hash_layer(
//                 layouter.namespace(|| format!("hash level {}", level)),
//                 pair.0,
//                 pair.1,
//                 level
//             )?;
//         }

//         Ok(node)
//     }
// }