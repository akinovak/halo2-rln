use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error},
    plonk,
    // pasta::Fp
};

use pasta_curves::{
    pallas,
};

use crate:: {
    utils::{UtilitiesInstructions, NumericCell},
    // gadget::{
    //     swap::{SwapChip, SwapConfig, SwapInstruction},
    //     // merkle::{MerkleChip, Config}
    // }
};

// Absolute offsets for public inputs.
pub const MUX_OUTPUT: usize = 0;

#[derive(Clone, Debug)]
pub struct Config {
    advice: [Column<Advice>; 3],
    instance: Column<Instance>,
}


#[derive(Debug, Default)]
pub struct Circuit {
}

impl UtilitiesInstructions<pallas::Base> for Circuit {
    type Var = NumericCell<pallas::Base>;
}

impl plonk::Circuit<pallas::Base> for Circuit {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {

        let advice = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        for advice in advice.iter() {
            meta.enable_equality((*advice).into());
        }


        Config {
            advice, 
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let config = config.clone();

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     // use halo2::{
//     //     dev::MockProver,
//     //     pasta::Fp
//     // };

//     // use super::Circuit;
// }