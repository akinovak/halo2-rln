// use halo2::{
//     circuit::{Chip, Layouter},
//     arithmetic::FieldExt,
//     plonk::{Advice, Column, ConstraintSystem, Error},
// };
// use std::{marker::PhantomData};
// use crate::utils::{NumericCell, UtilitiesInstructions};
// use crate::gadget::swap::{SwapConfig, SwapChip, SwapInstruction};

// use super::{MerkleInstructions};

// #[derive(Clone, Debug)]
// pub struct MerkleConfig {
//     swap_config: SwapConfig,
//     advices: [Column<Advice>; 3]
// }


// #[derive(Clone, Debug)]
// pub struct MerkleChip<F> {
//     config: MerkleConfig,
//     _marker: PhantomData<F>,
// }

// impl<F: FieldExt> Chip<F> for MerkleChip<F> {
//     type Config = MerkleConfig;
//     type Loaded = ();

//     fn config(&self) -> &Self::Config {
//         &self.config
//     }

//     fn loaded(&self) -> &Self::Loaded {
//         &()
//     }
// }

// impl<F: FieldExt> UtilitiesInstructions<F> for MerkleChip<F> {
//     type Var = CellValue<F>;
// } 

// impl<F: FieldExt> MerkleChip<F> {
//     pub fn configure(
//         meta: &mut ConstraintSystem<F>,
//         advices: [Column<Advice>; 3],
//     ) -> MerkleConfig {

//         let swap_config = SwapChip::configure(meta, advices);

//         let config = MerkleConfig {
//             swap_config,
//             advices
//         };

//         config
//     }

//     pub fn construct(config: MerkleConfig) -> Self {
//         MerkleChip {
//             config, 
//             _marker: PhantomData
//         }
//     }
// }

// impl<F: FieldExt> SwapInstruction<F> for MerkleChip<F> {
//     fn swap(
//         &self,
//         layouter: impl Layouter<F>,
//         pair: (Self::Var, Option<F>),
//         swap: Option<bool>,
//     ) -> Result<(Self::Var, Self::Var), Error> {
//         let config = self.config().swap_config.clone();
//         let chip = SwapChip::<F>::construct(config);
//         chip.swap(layouter, pair, swap)
//     }
// }

// impl<F: FieldExt> MerkleInstructions<F> for MerkleChip<F> {
//     fn hash_layer(
//         &self, 
//         mut layouter: impl Layouter<F>,
//         left: Self::Var,
//         right: Self::Var,
//         level: usize,
//     ) -> Result<Self::Var, Error> {

//         let config = self.config.clone();

//         let mocked_hash_value = Some(left.value.unwrap() - right.value.unwrap());

//         layouter.assign_region(
//             || "witness root",
//             |mut region| {
//                 let row_offset = 0;
//                 let cell = region.assign_advice(
//                     || "root",
//                     config.advices[level % 3],
//                     row_offset,
//                     || mocked_hash_value.ok_or(Error::Synthesis),
//                 )?;
                
//                 let inter_root = CellValue {
//                     cell, 
//                     value: mocked_hash_value
//                 };

//                 Ok(inter_root)
//             }
//         )
//     }
// }