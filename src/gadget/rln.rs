use crate::halo2::{
    circuit::Layouter,
    plonk::Error,
    arithmetic::FieldExt
};

pub mod chip;

use crate::utils::{UtilitiesInstructions};
pub use chip::{RlnConfig, RlnChip};

pub(crate) trait RlnInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    fn calculate_output(
        &self,
        layouter: impl Layouter<F>,
        private_key: Self::Var, 
        epoch: Self::Var,
        signal: Self::Var,
    ) -> Result<(Self::Var, Self::Var), Error>;

    fn calculate_nullifier(
        &self, 
        layouter: impl Layouter<F>,
        k: Self::Var
    ) -> Result<Self::Var, Error>;
}