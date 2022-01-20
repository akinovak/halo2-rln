use criterion::{criterion_group, criterion_main, Criterion};

extern crate rln;
use crate::rln::{
    merkle::IncrementalTree,
    client::{calculate_output},
    proof::{Proof, Instance as PublicInstance},
    keys::{ProvingKey}
};

use rand;
use std::convert::TryInto;
use ff::Field;
use halo2::{
    dev::MockProver,
    pasta::Fp,
    circuit::{Layouter, SimpleFloorPlanner},
    plonk,
    arithmetic::FieldExt,
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof,
        Error, Advice, Instance, Column, ConstraintSystem
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};

use std::marker::PhantomData;
use pasta_curves::{pallas, vesta};

use crate::rln::utils::{UtilitiesInstructions, NumericCell, Numeric};
use rln::gadget::merkle::{MerkleChip, MerkleConfig, InclusionProof};
use crate::rln::poseidon::{P128Pow5T3};
use crate::rln::gadget::poseidon::{Pow5T3Chip as PoseidonChip};

const DEPTH: usize = 30;

#[derive(Clone, Debug)]
pub struct Config<F: FieldExt> {
    advice: [Column<Advice>; 4],
    instance: Column<Instance>,
    merkle_config: MerkleConfig<F>
}


#[derive(Debug, Default, Clone)]
pub struct Circuit {
    leaf: Option<Fp>,
    siblings: [Option<Fp>; DEPTH],
    root: Option<Fp>,
    leaf_pos: [Option<bool>; DEPTH]
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

fn bench_merkle(c: &mut Criterion) {
    let k = 11u32;

    let mut tree = IncrementalTree::new(Fp::zero(), DEPTH);

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

    let params = halo2::poly::commitment::Params::<vesta::Affine>::new(k);
    let empty_circuit: Circuit = Default::default();

    let vk = plonk::keygen_vk(&params, &empty_circuit).unwrap();
    let pk = plonk::keygen_pk(&params, vk, &empty_circuit).unwrap();

    let circuit = Circuit {
        leaf: Some(leaf),
        siblings: siblings.try_into().expect("siblings with incorrect length"),
        root: Some(tree.root()), 
        leaf_pos: pos.try_into().expect("siblings with incorrect length")
    };

    let mut group = c.benchmark_group("merkle-proof");
    group.sample_size(10);
    group.bench_function("merkle", |b| {
        b.iter(|| {
            let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
            plonk::create_proof(&params, &pk, &[circuit.clone()], &[&[&[]]], &mut transcript)
            .expect("proof generation should not fail")
        });
    });

}

fn criterion_benchmark(c: &mut Criterion) {
    bench_merkle(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);