use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error},
    plonk,
    pasta::Fp
};

use pasta_curves::{
    pallas,
};

use crate:: {
    utils::{UtilitiesInstructions, NumericCell, CellValue, Numeric, Var, from_cell_vale_to_numeric},
    gadget::{
        poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig, Hash as PoseidonHash},
        // merkle::{MerkleChip, Config}
    },
    poseidon::{ConstantLength, P128Pow5T3}
};

// Absolute offsets for public inputs.
pub const MUX_OUTPUT: usize = 0;

#[derive(Clone, Debug)]
pub struct Config {
    advice: [Column<Advice>; 4],
    instance: Column<Instance>,
    poseidon_config: PoseidonConfig<Fp>
}


#[derive(Debug, Default)]
pub struct Circuit {
    a: Option<Fp>
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


        Config {
            advice, 
            instance,
            poseidon_config
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let config = config.clone();

        let a = self.load_private(
            layouter.namespace(|| "witness identity_trapdoor"),
            config.advice[0],
            self.a,
        )?;

        let poseidon_config = config.poseidon_config;
        let poseidon_chip = PoseidonChip::construct(poseidon_config);
        let mut poseidon_hasher: PoseidonHash
        <
            Fp, 
            PoseidonChip<Fp>, 
            P128Pow5T3, 
            ConstantLength<1_usize>, 
            3_usize, 
            2_usize
        > 
            = PoseidonHash::init(poseidon_chip, layouter.namespace(|| "init hasher"), ConstantLength::<1>)?;

        // let loaded_message = poseidon_hasher.witness_message_pieces(
        //     config.poseidon_config,
        //     layouter.namespace(|| format!("witnessing: {}", to_hash)),
        //     message
        // )?;

        let message = [a; 1];

        let word = poseidon_hasher.hash(layouter.namespace(|| "wtns"), message)?;
        let digest: CellValue<Fp> = word.inner().into();

        let assigned = from_cell_vale_to_numeric(layouter.namespace(|| "dummy conf"), config.advice[0], digest.value())?;

        println!("{:?}", assigned.value());

        Ok(())
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

    use pasta_curves::pallas;

    use super::{Circuit};

    use crate::utils::{UtilitiesInstructions, NumericCell, Numeric};

    use crate::poseidon::{Hash, P128Pow5T3, ConstantLength};

    #[test]
    fn rln_test() {
        let k = 8;
    
        let circuit = Circuit {
            a: Some(Fp::from(1)),
        };

        let hashed = Hash::init(P128Pow5T3, ConstantLength::<1>).hash([Fp::from(1)]);
        println!("{:?}", hashed);
        
        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}