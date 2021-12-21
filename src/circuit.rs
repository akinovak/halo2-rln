use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error},
    plonk,
    pasta::Fp
};
use std::marker::PhantomData;
use pasta_curves::pallas;

use crate:: {
    utils::{UtilitiesInstructions, NumericCell, CellValue, Var, from_cell_vale_to_numeric},
    gadget::{
        poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig, Hash as PoseidonHash},
        rln::{RlnChip, RlnConfig, RlnInstructions},
        merkle::{MerkleChip, MerkleConfig, InclusionProof}
    },
    poseidon::{ConstantLength, P128Pow5T3}
};

// Absolute offsets for public inputs.
pub const Y: usize = 0;
pub const NULLIFIER: usize = 1;
pub const SIGNAL: usize = 2;
pub const ROOT: usize = 3;

#[derive(Clone, Debug)]
pub struct Config {
    advice: [Column<Advice>; 4],
    instance: Column<Instance>,
    poseidon_config: PoseidonConfig<Fp>,
    merkle_config: MerkleConfig<Fp>,
    rln_config: RlnConfig<Fp>
}


#[derive(Debug, Default)]
pub struct Circuit {
    secret: Option<Fp>,
    signal: Option<Fp>,
    siblings: [Option<Fp>; 3],
    pos: [Option<bool>; 3],
    epoch: Option<Fp>
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
        let rln_config = RlnChip::configure(meta, advice[0..3].try_into().unwrap(), poseidon_config.clone());
        let merkle_config = MerkleChip::<pallas::Base>::configure(meta, advice[0..3].try_into().unwrap(), poseidon_config.clone());

        Config {
            advice, 
            instance,
            poseidon_config,
            merkle_config,
            rln_config
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let config = config.clone();

        let secret = self.load_private(
            layouter.namespace(|| "witness identity_trapdoor"),
            config.advice[0],
            self.secret,
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
        let y = rln_chip.calculate_y(layouter.namespace(|| "calculate y"), secret.clone(), epoch, signal.clone())?;
        let nullifier = rln_chip.calculate_nullifier(layouter.namespace(|| "calculate nullifier"), y.clone())?;

        let poseidon_config = config.poseidon_config;
        let poseidon_chip = PoseidonChip::construct(poseidon_config);
        let poseidon_hasher: PoseidonHash
        <
            Fp, 
            PoseidonChip<Fp>, 
            P128Pow5T3, 
            ConstantLength<1_usize>, 
            3_usize, 
            2_usize
        > 
            = PoseidonHash::init(poseidon_chip, layouter.namespace(|| "init hasher"), ConstantLength::<1>)?;

        let message = [secret; 1];

        let word = poseidon_hasher.hash(layouter.namespace(|| "wtns"), message)?;
        let digest: CellValue<Fp> = word.inner().into();
        let commitment = from_cell_vale_to_numeric(layouter.namespace(|| "dummy conf"), config.advice[0], digest.value())?;

        let chip = MerkleChip::construct(config.merkle_config.clone());
        let inclusion_proof = InclusionProof {
            merkle_chip: chip,
            siblings: self.siblings,
            leaf_pos: self.pos,
            _marker: PhantomData::<pallas::Base>
        };

        let root = 
        inclusion_proof.calculate_root(
            layouter.namespace(|| "merkle root"),
            commitment
        )?;

        self.expose_public(layouter.namespace(|| "expose y"), config.instance, y, Y)?;
        self.expose_public(layouter.namespace(|| "expose nullifier"), config.instance, nullifier, NULLIFIER)?;
        self.expose_public(layouter.namespace(|| "expose signal"), config.instance, signal, SIGNAL)?;
        self.expose_public(layouter.namespace(|| "expose root"), config.instance, root, ROOT)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use halo2::{
        dev::MockProver,
        pasta::Fp,
    };
    use super::{Circuit};
    use crate::poseidon::{Hash, P128Pow5T3, ConstantLength};
    use crate::merkle::IncrementalTree;
    use rand;
    use std::convert::TryInto;
    use ff::Field;

    #[test]
    fn round_trip() {
        let mut rng = rand::thread_rng();
        let mut tree = IncrementalTree::new(Fp::zero(), 3);
        let k = 10;

        let secret = Fp::random(&mut rng);
        let commitment = Hash::init(P128Pow5T3, ConstantLength::<1>).hash([secret]);

        let num_of_leaves = 5;

        for _ in 0..num_of_leaves {
            tree.insert(Fp::random(&mut rng));
        }

        tree.insert(commitment);

        let (siblings, pos) = tree.witness(commitment);
        let pos: Vec<Option<bool>> = pos.iter().map(|pos| Some(*pos)).collect();
        let siblings: Vec<Option<Fp>> = siblings.iter().map(|sibling| Some(*sibling)).collect();

        let epoch = Fp::random(&mut rng);
        let signal = Fp::random(&mut rng);
    
        let circuit = Circuit {
            secret: Some(secret),
            signal: Some(signal),
            siblings: siblings.try_into().expect("siblings with incorrect length"),
            pos: pos.try_into().expect("pos with incorrect length"),
            epoch: Some(epoch)
        };

        let coef = Hash::init(P128Pow5T3, ConstantLength::<2>).hash([secret, epoch]);
        let y = coef * signal + secret;
        let nullifier = Hash::init(P128Pow5T3, ConstantLength::<1>).hash([y]);

        let public_inputs = vec![y, nullifier, signal, tree.root()];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}