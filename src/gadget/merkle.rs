use crate::halo2::{
    plonk::{Error},
    circuit::{Layouter}
};
use pasta_curves::pallas;
mod chip;

use std::marker::PhantomData;

use crate::gadget::swap::{SwapInstruction};
pub use chip::{MerkleConfig, MerkleChip};
use crate::utils::{UtilitiesInstructions};


pub trait MerkleInstructions: UtilitiesInstructions<pallas::Base> {
    fn hash_layer(
        &self, 
        layouter: impl Layouter<pallas::Base>,
        left: Self::Var,
        right: Self::Var,
        level: usize
    ) -> Result<Self::Var, Error>;
}

#[derive(Clone, Debug)]
pub struct InclusionProof<
    const DEPTH: usize,
>
{
    pub merkle_chip: MerkleChip<pallas::Base>,
    pub siblings: [Option<pallas::Base>; DEPTH],
    pub leaf_pos: [Option<bool>; DEPTH],
    pub _marker: PhantomData<pallas::Base>,
}

impl
<
    const DEPTH: usize,
>  InclusionProof<DEPTH>
{
    pub fn calculate_root(
        &self, 
        mut layouter: impl Layouter<pallas::Base>,
        leaf: <MerkleChip<pallas::Base> as UtilitiesInstructions<pallas::Base>>::Var,
    ) -> Result<<MerkleChip<pallas::Base> as UtilitiesInstructions<pallas::Base>>::Var, Error> {

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
    use crate::halo2::{
        dev::MockProver,
        pasta::Fp,
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Advice, Instance, Column, ConstraintSystem, Error},
        plonk,
        arithmetic::FieldExt,
    };
    use std::convert::TryInto;
    use std::marker::PhantomData;
    use pasta_curves::pallas;

    use crate::utils::{UtilitiesInstructions, NumericCell, Numeric};
    use super::{MerkleChip, MerkleConfig, InclusionProof};
    use crate::poseidon::{P128Pow5T3};
    use crate::gadget::poseidon::{Pow5T3Chip as PoseidonChip};

    use crate::merkle::IncrementalTree;


    #[derive(Clone, Debug)]
    pub struct Config<F: FieldExt> {
        advice: [Column<Advice>; 4],
        instance: Column<Instance>,
        merkle_config: MerkleConfig<F>
    }


    #[derive(Debug, Default)]
    pub struct Circuit {
        leaf: Option<Fp>,
        siblings: [Option<Fp>; 10],
        root: Option<Fp>,
        leaf_pos: [Option<bool>; 10]
    }

    impl UtilitiesInstructions<pallas::Base> for Circuit {
        type Var = NumericCell<pallas::Base>;
    }

    impl plonk::Circuit<pallas::Base> for Circuit {
        type Config = Config<pallas::Base>;
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

            let merkle_config = MerkleChip::<pallas::Base>::configure(meta, advice[0..3].try_into().unwrap(), poseidon_config);

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

            Ok({})
        }
    }

    #[test]
    fn inclusion_proof_test() {
        let k = 9;
        let depth = 10;

        let mut tree = IncrementalTree::new(Fp::zero(), depth);

        tree.insert(Fp::from(2));
        tree.insert(Fp::from(3));
        tree.insert(Fp::from(4));
        tree.insert(Fp::from(5));
        tree.insert(Fp::from(6));
        tree.insert(Fp::from(7));

        let leaf = Fp::from(6);
        let (siblings, pos) = tree.witness(leaf);

        let pos: Vec<Option<bool>> = pos.iter().map(|pos| Some(*pos)).collect();
        let siblings: Vec<Option<Fp>> = siblings.iter().map(|sibling| Some(*sibling)).collect();


        let circuit = Circuit {
            leaf: Some(leaf),
            siblings: siblings.try_into().expect("siblings with incorrect length"),
            root: Some(tree.root()), 
            leaf_pos: pos.try_into().expect("siblings with incorrect length")
        };

        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}