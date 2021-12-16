use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
    arithmetic::FieldExt,
    poly::Rotation
};

pub mod chip;

use std::{array, marker::PhantomData};
use crate::utils::{NumericCell, UtilitiesInstructions, Numeric};

pub(crate) trait RlnInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    fn calculate_y(
        &self,
        layouter: impl Layouter<F>,
        private_key: Self::Var, 
        epoch: Self::Var,
        signal: Self::Var,
    ) -> Result<Self::Var, Error>;
}


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