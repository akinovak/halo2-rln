extern crate halo2;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
    pasta::Fp
};

use pasta_curves::pallas;
use std::marker::PhantomData;
use std::array;

use super::RlnInstructions;

use crate::{
    utils::{NumericCell, Numeric, UtilitiesInstructions, from_cell_vale_to_numeric, CellValue, Var},
    gadget::poseidon::{HashInstruction, Pow5T3Config as PoseidonConfig, Pow5T3Chip as PoseidonChip, Hash},
    poseidon::{ConstantLength, P128Pow5T3}
};

#[derive(Clone, Debug)]
pub struct RlnConfig<F: FieldExt> {
    n: Column<Advice>,
    k: Column<Advice>,
    x: Column<Advice>,
    q_rln: Selector,
    poseidon_config: PoseidonConfig<F>
}

#[derive(Clone, Debug)]
pub struct RlnChip<F: FieldExt> {
    config: RlnConfig<F>,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for RlnChip<F> {
    type Config = RlnConfig<F>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> UtilitiesInstructions<F> for RlnChip<F> {
    type Var = NumericCell<F>;
}


impl<F: FieldExt> RlnChip<F> {
    pub fn construct(config: RlnConfig::<F>) -> Self {
        RlnChip {
            config, 
            _marker: PhantomData
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
        poseidon_config: PoseidonConfig<F>
    ) -> <Self as Chip<F>>::Config {

        let n = advice[0];
        let k = advice[1];
        let x = advice[2];

        let q_rln = meta.selector();

        let config = RlnConfig {
            n,
            k,
            x,
            q_rln,
            poseidon_config: poseidon_config.clone()
        };

        meta.create_gate("constraint rln", |meta| {
            let q_rln = meta.query_selector(q_rln);
            let n = meta.query_advice(config.n, Rotation::cur());
            let k = meta.query_advice(config.k, Rotation::cur());
            let x = meta.query_advice(config.x, Rotation::cur());

            let y = meta.query_advice(config.n, Rotation::next());

            let linearity_check = y - k*x - n;

            array::IntoIter::new([linearity_check])
            .map(move |poly| q_rln.clone() * poly)
        });

        config
    }
}

impl<const LEN: usize> HashInstruction<pallas::Base, LEN> for RlnChip<pallas::Base> {
    fn hash(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        message: [Self::Var; LEN],
    ) -> Result<Self::Var, Error> {
        let config = self.config().clone();
        let poseidon_config = config.poseidon_config.clone();
        let chip = PoseidonChip::<pallas::Base>::construct(poseidon_config);

        let poseidon_hasher: Hash
        <
            Fp, 
            PoseidonChip<Fp>, 
            P128Pow5T3, 
            ConstantLength<LEN>, 
            3_usize, 
            2_usize
        >  = Hash::init(chip, layouter.namespace(|| "init hasher"), ConstantLength::<LEN>)?;
        let word = poseidon_hasher.hash(layouter.namespace(|| "digest message"), message)?;
        let digest: CellValue<pallas::Base> = word.inner().into();
        let assigned = from_cell_vale_to_numeric(layouter.namespace(|| "dummy"), config.n, digest.value())?;
        Ok(assigned)
    }
}

impl RlnInstructions<pallas::Base> for RlnChip<pallas::Base> {
    fn calculate_y(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        private_key: Self::Var, 
        epoch: Self::Var,
        signal: Self::Var,
    ) -> Result<Self::Var, Error> {
        let config = self.config();

        let hashed = self.hash(layouter.namespace(|| "hash to k"), [private_key.clone(), epoch])?;

        layouter.assign_region(
            || "rln", 
            |mut region| {
                let mut row_offset = 0;

                let n = private_key.copy(|| "copy pk", &mut region, config.n, row_offset)?;
                let x = signal.copy(|| "copy x", &mut region, config.x, row_offset)?;

                config.q_rln.enable(&mut region, row_offset)?;

                let k = {
                    let cell = region.assign_advice(
                        || "witness k",
                        config.k,
                        row_offset,
                        || hashed.value().ok_or(Error::Synthesis),
                    )?;
                    NumericCell::new(cell)
                };

                row_offset += 1;
                let y = {
                    let y = k
                        .value()
                        .zip(x.value())
                        .zip(n.value())
                        .map(|((k, x), n)| k * x + n);
                    // let y = k.value();
                    let cell = region.assign_advice(
                        || "witness k",
                        config.n,
                        row_offset,
                        || y.ok_or(Error::Synthesis),
                    )?;
                    NumericCell::new(cell)
                };

                Ok(y)
            }
        )
    }

    fn calculate_nullifier(
        &self, 
        mut layouter: impl Layouter<pallas::Base>,
        y: Self::Var
    ) -> Result<Self::Var, Error> {
        let config = self.config().clone();
        let nullifier = self.hash(layouter.namespace(|| "calculate nullifier"), [y])?;

        layouter.assign_region(
            || "witness nullifier",
            |mut region| {
                let row_offset = 0;
                let cell = region.assign_advice(
                    || "nullifier",
                    config.n,
                    row_offset,
                    || nullifier.value().ok_or(Error::Synthesis),
                )?;

                Ok(NumericCell::new(cell))
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
        plonk::{Advice, Column, ConstraintSystem, Error, Instance},
        plonk,
    };

    use pasta_curves::pallas;

    use super::{RlnChip, RlnConfig, RlnInstructions};

    use crate::utils::{UtilitiesInstructions, NumericCell};
    use crate::poseidon::{ConstantLength, P128Pow5T3, Hash};
    use crate::gadget::poseidon::{Pow5T3Chip as PoseidonChip};

    #[derive(Clone, Debug)]
    pub struct Config {
        advice: [Column<Advice>; 4],
        instance: Column<Instance>,
        rln_config: RlnConfig<pallas::Base>
    }


    #[derive(Debug, Default)]
    pub struct Circuit {
        private_key: Option<Fp>,
        epoch: Option<Fp>,
        signal: Option<Fp>,
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
            meta.advice_column()
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        for advice in advice.iter() {
            meta.enable_equality((*advice).into());
        }

        let rc_a = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let rc_b = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];

        meta.enable_constant(rc_b[0]);

        let poseidon_config = PoseidonChip::configure(meta, P128Pow5T3, advice[0..3].try_into().unwrap(), advice[3], rc_a, rc_b);
        let rln_config = RlnChip::<pallas::Base>::configure(meta, advice[..3].try_into().unwrap(), poseidon_config);

            Config {
                advice,
                instance,
                rln_config
            }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error>{
            let config = config.clone();

            let private_key = self.load_private(
                layouter.namespace(|| "witness identity_trapdoor"),
                config.advice[0],
                self.private_key,
            )?;

            let epoch = self.load_private(
                layouter.namespace(|| "witness identity_trapdoor"),
                config.advice[0],
                self.epoch,
            )?;

            let signal = self.load_private(
                layouter.namespace(|| "witness identity_trapdoor"),
                config.advice[0],
                self.signal,
            )?;

            let rln_chip = RlnChip::construct(config.rln_config);
            let y = rln_chip.calculate_y(layouter.namespace(|| "calculate y"), private_key, epoch, signal.clone())?;
            let nullifier = rln_chip.calculate_nullifier(layouter.namespace(|| "calculate nullifier"), y.clone())?;

            self.expose_public(layouter.namespace(|| "expose y"), config.instance, y, 0)?;
            self.expose_public(layouter.namespace(|| "expose nullifier"), config.instance, nullifier, 1)?;
            self.expose_public(layouter.namespace(|| "expose signal"), config.instance, signal, 2)?;

            Ok(())
        }
    }

    #[test]
    fn rln_test() {
        let k = 9;

        let private_key = Some(Fp::from(5));
        let epoch = Some(Fp::from(2));
        let signal = Some(Fp::from(3));
    
        let circuit = Circuit {
            private_key,
            epoch,
            signal
        };

        let coef = Hash::init(P128Pow5T3, ConstantLength::<2>).hash([private_key.unwrap(), epoch.unwrap()]);
        let y = coef * signal.unwrap() + private_key.unwrap();
        let nullifier = Hash::init(P128Pow5T3, ConstantLength::<1>).hash([y]);

        let public_inputs = vec![y, nullifier, signal.unwrap()];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}