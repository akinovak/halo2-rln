// /* 
// This chip should determine which value should be left and which right input to hasher
// based on leaf position at current level

// This chip considers that left value is already witnessed:
// first layer -> leaf is witnessed as private input
// ith layer -> left inter root is witnessed after hash layer
// */
// use halo2::{
//     circuit::{Chip, Layouter},
//     plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
//     arithmetic::FieldExt,
//     poly::Rotation
// };
// use std::{array, marker::PhantomData};
// use crate::utils::{NumericCell, UtilitiesInstructions, Numeric};

// pub trait RlnInstruction<F: FieldExt>: UtilitiesInstructions<F> {
// }

// #[derive(Clone, Debug)]
// pub struct RlnConfig {
// }


// #[derive(Clone, Debug)]
// pub struct RlnChip {
// }

// impl<F: FieldExt> Chip<F> for RlnChip {
//     type Config = RlnConfig;
//     type Loaded = ();

//     fn config(&self) -> &Self::Config {
//         &self.config
//     }

//     fn loaded(&self) -> &Self::Loaded {
//         &()
//     }
// }

// impl<F: FieldExt> UtilitiesInstructions<F> for RlnChip {
//     type Var = NumericCell<F>;
// }

// impl<F: FieldExt> RlnChip {
//     pub fn configure(
//         meta: &mut ConstraintSystem<F>,
//     ) -> RlnConfig {
//         let config = RlnConfig {
//         };

//         config
//     }

//     pub fn construct(config: RlnConfig) -> Self {
//         RlnConfig {
//             config, 
//             _marker: PhantomData
//         }
//     }
// }

// impl<F: FieldExt> RlnInstruction<F> for RlnChip {
// }

// #[cfg(test)]
// mod test {
//     use halo2::{
//         dev::MockProver,
//         pasta::Fp,
//         circuit::{Layouter, SimpleFloorPlanner},
//         plonk::{Advice, Instance, Column, ConstraintSystem, Error},
//         plonk,
//     };

//     use pasta_curves::pallas;

//     use super::{RlnChip, RlnConfig, RlnInstruction};

//     use crate::utils::{UtilitiesInstructions, NumericCell, Numeric};

//     #[derive(Clone, Debug)]
//     pub struct Config {
//         rln_config: RlnConfig
//     }


//     #[derive(Debug, Default)]
//     pub struct Circuit {
//     }

//     impl UtilitiesInstructions<pallas::Base> for Circuit {
//         type Var = NumericCell<pallas::Base>;
//     }

//     impl plonk::Circuit<pallas::Base> for Circuit {
//         type Config = Config;
//         type FloorPlanner = SimpleFloorPlanner;

//         fn without_witnesses(&self) -> Self {
//             Self::default()
//         }

//         fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
//             let rln_config = RlnChip::configure(meta);

//             Config {
//                 rln_config
//             }
//         }

//         fn synthesize(
//             &self,
//             config: Self::Config,
//             mut layouter: impl Layouter<pallas::Base>,
//         ) -> Result<(), Error> {
//             Ok(())
//         }
//     }

//     #[test]
//     fn swap_test() {
//         let k = 4;
    
//         let circuit = Circuit {
//         };

//         let public_inputs = vec![];
//         let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
//         assert_eq!(prover.verify(), Ok(()));
//     }
// }
