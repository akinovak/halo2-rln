use halo2::{
    plonk::{Error},
    circuit::{Layouter},
    arithmetic::FieldExt,
};
mod chip;

use std::marker::PhantomData;

use crate::gadget::swap::{SwapInstruction};
pub use chip::{MerkleConfig, MerkleChip};
use crate::utils::{UtilitiesInstructions};


pub trait MerkleInstructions<F:FieldExt>: UtilitiesInstructions<F> {
    fn hash_layer(
        &self, 
        layouter: impl Layouter<F>,
        left: Self::Var,
        right: Self::Var,
        level: usize
    ) -> Result<Self::Var, Error>;
}

#[derive(Clone, Debug)]
pub struct InclusionProof<
    F: FieldExt,
    const DEPTH: usize,
>
{
    pub merkle_chip: MerkleChip<F>,
    pub siblings: [Option<F>; DEPTH],
    pub leaf_pos: [Option<bool>; DEPTH],
    pub _marker: PhantomData<F>,
}

impl
<
    F: FieldExt,
    const DEPTH: usize,
>  InclusionProof<F, DEPTH>
{
    pub fn calculate_root(
        &self, 
        mut layouter: impl Layouter<F>,
        leaf: <MerkleChip<F> as UtilitiesInstructions<F>>::Var,
    ) -> Result<<MerkleChip<F> as UtilitiesInstructions<F>>::Var, Error> {

        let mut node = leaf;

        let chips = vec![self.merkle_chip.clone(); DEPTH];

        for (level, ((sibling, pos), chip)) in self.siblings.iter().zip(self.leaf_pos.iter()).zip(chips).enumerate() {
            let pair = {
                let pair = (node, *sibling);
                chip.swap(layouter.namespace(|| format!("swap pair on level {}", level)), pair, *pos)?
            };

            node = chip.hash_layer(
                layouter.namespace(|| format!("hash level {}", level)),
                pair.0,
                pair.1,
                level
            )?;
        }

        Ok(node)
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
    use std::marker::PhantomData;
    use pasta_curves::pallas;

    // use crate::gadget::swap::{SwapChip, SwapConfig, SwapInstruction};
    use crate::utils::{UtilitiesInstructions, NumericCell, Numeric};
    use super::{MerkleChip, MerkleConfig, InclusionProof};

    #[derive(Clone, Debug)]
    pub struct Config {
        advice: [Column<Advice>; 3],
        instance: Column<Instance>,
        merkle_config: MerkleConfig
    }


    #[derive(Debug, Default)]
    pub struct Circuit {
        leaf: Option<Fp>,
        siblings: [Option<Fp>; 3],
        root: Option<Fp>,
        leaf_pos: [Option<bool>; 3]
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

            let merkle_config = MerkleChip::configure(meta, advice);

            Config {
                advice, 
                instance,
                merkle_config
            }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            let config = config.clone();

            let leaf = self.load_private(
                layouter.namespace(|| "witness leaf"),
                config.advice[0],
                self.leaf,
            )?;

            let chip = MerkleChip::construct(config.merkle_config.clone());
            let inclusion_proof = InclusionProof {
                merkle_chip: chip,
                siblings: self.siblings,
                leaf_pos: self.leaf_pos,
                _marker: PhantomData::<pallas::Base>
            };

            let root = 
                inclusion_proof.calculate_root(
                    layouter.namespace(|| "merkle root"),
                    leaf
                )?;

            println!("{:?}", root.value());

            Ok({})
        }
    }

    #[test]
    fn inclusion_proof_test() {
        let k = 4;
    
        let circuit = Circuit {
            leaf: Some(Fp::from(100)),
            siblings: [Some(Fp::from(1)); 3],
            root: Some(Fp::from(2)), 
            leaf_pos: [Some(false); 3]
        };

        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}