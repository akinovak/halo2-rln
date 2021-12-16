extern crate halo2;

use halo2::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Chip, Layouter, Region, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance, Selector},
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
struct RlnConfig<F: FieldExt> {
    private_key: Column<Advice>,
    k: Column<Advice>,
    x: Column<Advice>,
    q_rln: Selector,
    poseidon_config: PoseidonConfig<F>
}

#[derive(Clone, Debug)]
struct RlnChip<F: FieldExt> {
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

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
        poseidon_config: PoseidonConfig<F>
    ) -> <Self as Chip<F>>::Config {

        let private_key = advice[0];
        let k = advice[1];
        let x = advice[2];

        let q_rln = meta.selector();

        let config = RlnConfig {
            private_key,
            k,
            x,
            q_rln,
            poseidon_config: poseidon_config.clone()
        };

        meta.create_gate("constraint swap", |meta| {
            let q_rln = meta.query_selector(q_rln);
            let n = meta.query_advice(config.private_key, Rotation::cur());
            let k = meta.query_advice(config.k, Rotation::cur());
            let x = meta.query_advice(config.x, Rotation::cur());

            let y = meta.query_advice(config.private_key, Rotation::next());

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
        let assigned = from_cell_vale_to_numeric(layouter.namespace(|| "dummy"), config.private_key, digest.value())?;
        Ok(assigned)
    }
}

// impl RlnInstructions<pallas::Base> for RlnChip<pallas::Base> {
//     fn calculate_y(
//         &self,
//         layouter: impl Layouter<pallas::Base>,
//         x: Self::Var, 
//         epoch: Option<pallas::Base>,
//     ) -> Result<Self::Var, Error> {

//     }
// }