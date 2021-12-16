use halo2::{
    circuit::{Chip, Layouter},
    arithmetic::FieldExt,
    plonk::{Advice, Column, ConstraintSystem, Error},
    pasta::Fp
};
use pasta_curves::pallas;

use std::{marker::PhantomData};
use crate::utils::{NumericCell, UtilitiesInstructions, Numeric, CellValue, Var, from_cell_vale_to_numeric};
use crate::gadget::swap::{SwapConfig, SwapChip, SwapInstruction};
use crate::gadget::poseidon::{HashInstruction, Pow5T3Config as PoseidonConfig, Pow5T3Chip as PoseidonChip, Hash};
use crate::poseidon::{P128Pow5T3, ConstantLength};

use super::{MerkleInstructions};

#[derive(Clone, Debug)]
pub struct MerkleConfig<F: FieldExt> {
    swap_config: SwapConfig,
    poseidon_config: PoseidonConfig<F>,
    advice: [Column<Advice>; 3]
}


#[derive(Clone, Debug)]
pub struct MerkleChip<F: FieldExt> {
    config: MerkleConfig<F>,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for MerkleChip<F> {
    type Config = MerkleConfig<F>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> UtilitiesInstructions<F> for MerkleChip<F> {
    type Var = NumericCell<F>;
} 

impl<F: FieldExt> MerkleChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
        poseidon_config: PoseidonConfig<F>
    ) -> MerkleConfig<F> {

        let swap_config = SwapChip::configure(meta, advice);

        let config = MerkleConfig {
            swap_config,
            poseidon_config: poseidon_config.clone(),
            advice
        };

        config
    }

    pub fn construct(config: MerkleConfig<F>) -> Self {
        MerkleChip {
            config, 
            _marker: PhantomData
        }
    }
}

impl<F: FieldExt> SwapInstruction<F> for MerkleChip<F> {
    fn swap(
        &self,
        layouter: impl Layouter<F>,
        pair: (Self::Var, Option<F>),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config().swap_config.clone();
        let chip = SwapChip::<F>::construct(config);
        chip.swap(layouter, pair, swap)
    }
}

impl<const L: usize> HashInstruction<pallas::Base, L> for MerkleChip<pallas::Base> {
    fn hash(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        message: [Self::Var; L],
    ) -> Result<Self::Var, Error> {
        let config = self.config().clone();
        let poseidon_config = config.poseidon_config.clone();
        let chip = PoseidonChip::<pallas::Base>::construct(poseidon_config);

        let poseidon_hasher: Hash
        <
            Fp, 
            PoseidonChip<Fp>, 
            P128Pow5T3, 
            ConstantLength<L>, 
            3_usize, 
            2_usize
        >  = Hash::init(chip, layouter.namespace(|| "init hasher"), ConstantLength::<L>)?;
        let word = poseidon_hasher.hash(layouter.namespace(|| "digest message"), message)?;
        let digest: CellValue<pallas::Base> = word.inner().into();
        let assigned = from_cell_vale_to_numeric(layouter.namespace(|| "dummy conf"), config.advice[0], digest.value())?;
        Ok(assigned)
    }
}

impl MerkleInstructions for MerkleChip<pallas::Base> {
    fn hash_layer(
        &self, 
        mut layouter: impl Layouter<pallas::Base>,
        left: Self::Var,
        right: Self::Var,
        level: usize,
    ) -> Result<Self::Var, Error> {

        let config = self.config.clone();
        let hashed = self.hash(layouter.namespace(|| format!("hashing: {}", level)), [left, right])?;

        layouter.assign_region(
            || "witness root",
            |mut region| {
                let row_offset = 0;
                let cell = region.assign_advice(
                    || "root",
                    config.advice[level % 3],
                    row_offset,
                    || hashed.value().ok_or(Error::Synthesis),
                )?;

                Ok(NumericCell::new(cell))
            }
        )
    }
}