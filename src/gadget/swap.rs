/* 
This chip should determine which value should be left and which right input to hasher
based on leaf position at current level

This chip considers that left value is already witnessed:
first layer -> leaf is witnessed as private input
ith layer -> left inter root is witnessed after hash layer
*/
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
    arithmetic::FieldExt,
    poly::Rotation
};
use std::{array, marker::PhantomData};
use crate::utils::{NumericCell, UtilitiesInstructions, Numeric};

pub trait SwapInstruction<F: FieldExt>: UtilitiesInstructions<F> {
    fn swap(
        &self,
        layouter: impl Layouter<F>,
        pair: (Self::Var, Option<F>),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error>;
}

#[derive(Clone, Debug)]
pub struct SwapConfig {
    pub q_swap: Selector,
    pub left: Column<Advice>,
    pub right: Column<Advice>,
    pub should_swap: Column<Advice>
}


#[derive(Clone, Debug)]
pub struct SwapChip<F> {
    config: SwapConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for SwapChip<F> {
    type Config = SwapConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> UtilitiesInstructions<F> for SwapChip<F> {
    type Var = NumericCell<F>;
}


//TODO remove last two advices
impl<F: FieldExt> SwapChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advices: [Column<Advice>; 3],
    ) -> SwapConfig {
        let left = advices[0];
        // we must enable equality so that copy can work
        meta.enable_equality(left.into());

        let q_swap = meta.selector();

        let config = SwapConfig {
            q_swap,
            left,
            right: advices[1],
            should_swap: advices[2],
        };

        meta.create_gate("constraint swap", |meta| {
            let q_swap = meta.query_selector(q_swap);
            
            let left = meta.query_advice(config.left, Rotation::cur());
            let right = meta.query_advice(config.right, Rotation::cur());

            let left_swapped = meta.query_advice(config.left, Rotation::next());
            let right_swapped = meta.query_advice(config.right, Rotation::next());

            let should_swap = meta.query_advice(config.should_swap, Rotation::cur());

            let one = Expression::Constant(F::one());

            let check_left = 
                left_swapped - right.clone() * should_swap.clone() - left.clone() * (one.clone() - should_swap.clone());

            let check_right = 
                right_swapped - left.clone() * should_swap.clone() - right.clone() * (one.clone() - should_swap.clone());
            
            let check_bool = should_swap.clone() * (one.clone() - should_swap.clone());

            // we should constraint all try polynomials
            array::IntoIter::new([check_left, check_right, check_bool])
                .map(move |poly| q_swap.clone() * poly)

        });

        config
    }

    pub fn construct(config: SwapConfig) -> Self {
        SwapChip {
            config, 
            _marker: PhantomData
        }
    }
}

impl<F: FieldExt> SwapInstruction<F> for SwapChip<F> {
    fn swap(
        &self, 
        mut layouter: impl Layouter<F>,
        pair: (Self::Var, Option<F>),
        swap: Option<bool>
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config();

        layouter.assign_region(
            || "swap", 
            |mut region| {
                let mut row_offset = 0;
                config.q_swap.enable(&mut region, 0)?;

                //make sure that root from previous level is equal to left
                let left = pair.0.copy(|| "copy left", &mut region, config.left, row_offset)?;

                let right = {
                    let cell = region.assign_advice(
                        || "witness right",
                        config.right,
                        row_offset,
                        || pair.1.ok_or(Error::Synthesis),
                    )?;
                    NumericCell::new(cell)
                };

                let swap_value = swap.map(|swap| F::from(swap as u64));
                region.assign_advice(
                    || "witness swap",
                    config.should_swap,
                    row_offset,
                    || swap_value.ok_or(Error::Synthesis),
                )?;

                row_offset += 1;

                let left_swapped = {

                    let swapped = left
                        .value()
                        .zip(right.value())
                        .zip(swap)
                        .map(|((left, right), swap)| if swap { right } else { left });

                    let cell = region.assign_advice(
                        || "witness left_swapped",
                        config.left,
                        row_offset,
                        || swapped.ok_or(Error::Synthesis),
                    )?;

                    NumericCell::new(cell)
                };

                let right_swapped = {

                    let swapped = left
                        .value()
                        .zip(right.value())
                        .zip(swap)
                        .map(|((left, right), swap)| if swap { left } else { right });

                    let cell = region.assign_advice(
                        || "witness right_swapped",
                        config.right,
                        row_offset,
                        || swapped.ok_or(Error::Synthesis)
                    )?;

                    NumericCell::new(cell)
                };  

                Ok((left_swapped, right_swapped))
            } 
        )
    }
}

#[cfg(test)]
mod test {
    use halo2::{
        dev::MockProver,
        pasta::Fp,
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Advice, Instance, Column, ConstraintSystem, Error},
        plonk,
    };

    use pasta_curves::pallas;

    use super::{SwapChip, SwapConfig, SwapInstruction};

    use crate::utils::{UtilitiesInstructions, NumericCell, Numeric};

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

            let swap_chip = SwapChip::<pallas::Base>::construct(config.swap_config.clone());
            let swapped_pair = swap_chip.swap(layouter.namespace(|| "calculate mux"), (a, self.b), self.should_swap)?;

            match self.should_swap.unwrap() {
                true => { 
                    assert_eq!(swapped_pair.0.value().unwrap(), self.b.unwrap());
                    assert_eq!(swapped_pair.1.value().unwrap(), self.a.unwrap());
                }, 
                false => {
                    assert_eq!(swapped_pair.0.value().unwrap(), self.a.unwrap());
                    assert_eq!(swapped_pair.1.value().unwrap(), self.b.unwrap());
                }
            };

            Ok({})
        }
    }

    #[test]
    fn swap_test() {
        let k = 4;
    
        let circuit = Circuit {
            a: Some(Fp::from(1)), 
            b: Some(Fp::from(2)), 
            should_swap: Some(false)
        };

        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
