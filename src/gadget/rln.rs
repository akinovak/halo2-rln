use halo2::{
    circuit::Layouter,
    plonk::Error,
    arithmetic::FieldExt
};

pub mod chip;

use crate::utils::{UtilitiesInstructions};

pub(crate) trait RlnInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    fn calculate_y(
        &self,
        layouter: impl Layouter<F>,
        private_key: Self::Var, 
        epoch: Self::Var,
        signal: Self::Var,
    ) -> Result<Self::Var, Error>;
}