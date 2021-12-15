use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error},
    plonk,
    pasta::Fp
};

use pasta_curves::{
    pallas,
};

use crate:: {
    utils::{UtilitiesInstructions, NumericCell},
    gadget::{
        swap::{SwapChip, SwapConfig, SwapInstruction}
    }
};

// Absolute offsets for public inputs.
pub const MUX_OUTPUT: usize = 0;

#[derive(Clone, Debug)]
pub struct Config {
    advice: [Column<Advice>; 3],
    instance: Column<Instance>,
    swap_config: SwapConfig
}


#[derive(Debug, Default)]
pub struct Circuit {
    a: Option<Fp>,
    b: Option<Fp>,
    should_swap: Option<bool>
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

        let swap_config = SwapChip::configure(meta, advice);

        Config {
            advice, 
            instance,
            swap_config
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let config = config.clone();

        let a = self.load_private(
            layouter.namespace(|| "witness a"),
            config.advice[0],
            self.a,
        )?;

        // let b = self.load_private(
        //     layouter.namespace(|| "witness b"),
        //     config.advice[0],
        //     self.b,
        // )?;

        // let selector = self.load_private(
        //     layouter.namespace(|| "witness selector"),
        //     config.advice[0],
        //     self.should_swap,
        // )?;

        let swap_chip = SwapChip::<pallas::Base>::construct(config.swap_config.clone());
        let pair = swap_chip.swap(layouter.namespace(|| "calculate mux"), (a, self.b), self.should_swap)?;

        println!("{:?}", pair.0);
        println!("{:?}", pair.1);

        Ok({})
    }
}

#[cfg(test)]
mod tests {
    use halo2::{
        dev::MockProver,
        pasta::Fp
    };

    use super::Circuit;

    #[test]
    fn full_test() {
        let k = 4;
        // let selector = Fp::from(0);
    
        let circuit = Circuit {
            a: Some(Fp::from(1)), 
            b: Some(Fp::from(2)), 
            should_swap: Some(false)
        };

        let mut public_inputs = vec![Fp::from(2)];

        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}