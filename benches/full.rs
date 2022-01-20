use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate rln;
use crate::rln::{
    circuit::{Circuit},
    merkle::IncrementalTree,
    client::{calculate_output},
    poseidon::{Hash, P128Pow5T3, ConstantLength},
    proof::{Proof, Instance},
    keys::{ProvingKey}
};

use pasta_curves::{pallas};

use rand;
use std::convert::TryInto;
use ff::Field;
use halo2::{
    pasta::Fp,
    transcript::{Blake2bWrite, Challenge255}
};

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n-1) + fibonacci(n-2),
//     }
// }

fn bench_rln(name: &str, depth: usize, c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut tree = IncrementalTree::new(Fp::zero(), depth);
    let k = 15;

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
    let msg = "hello rln";
    let (y, nullifier, signal) = calculate_output(secret, epoch, msg);

    let pk = ProvingKey::build(k);
    // let prover_name = name.to_string() + "-prover";

    let circuit = Circuit {
        secret: Some(secret),
        signal: Some(signal),
        siblings: siblings.clone().try_into().expect("siblings with incorrect length"),
        pos: pos.clone().try_into().expect("pos with incorrect length"),
        epoch: Some(epoch)
    };

    let instance = Instance {
        y, 
        nullifier,
        signal, 
        root: tree.root()
    };

    let mut group = c.benchmark_group("rln-proof");
    group.sample_size(10);
    group.bench_function("full", |b| {
        b.iter(|| {
            Proof::create(&pk, &[circuit.clone()], &[instance.clone()]).expect("proof should not fail")
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_rln("poseidon-3", 20, c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);