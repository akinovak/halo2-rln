use halo2::arithmetic::FieldExt;
use halo2::pasta::Fp;
use std::iter;
use pasta_curves::pallas;
use crate::poseidon::{self, P128Pow5T3, ConstantLength};

pub struct IncrementalTree {
    root: Fp,
    zeroes: Vec<Fp>,
    nodes: Vec<Vec<Fp>>,
    depth: usize,
    position: usize,
}

trait Hash<F: FieldExt> {
    fn hash_layer(
        left: F, 
        right: F
    ) -> F;
}

impl Hash<pallas::Base> for IncrementalTree {
    fn hash_layer(
        left: pallas::Base,
        right: pallas::Base
    ) -> pallas::Base {
        poseidon::Hash::init(P128Pow5T3, ConstantLength::<2>).hash([left, right])
    }
}

impl IncrementalTree {
    pub fn new(zero_value: Fp, depth: usize) -> Self {

        if depth > 32 { panic!("MAX DEPTH EXCEEDED") }

        let zeroes: Vec<Fp> = {
            iter::empty()
            .chain(Some(zero_value))
            .chain(
                (0..depth).scan(zero_value, |zero, _level| {
                    *zero = IncrementalTree::hash_layer(*zero, *zero);
                    Some(*zero)
                })
            )
            .collect()
        };

        assert_eq!(zeroes.len(), depth + 1);

        IncrementalTree {
            root: *zeroes.last().unwrap(),
            zeroes,
            nodes: vec![Vec::new(); depth],
            depth,
            position: 0,
        }
    }

    pub fn insert(&mut self, leaf: Fp) {
        if leaf == self.zeroes[0] {
            panic!("Leaf cannot be equal to zero value");
        }

        if self.nodes.len() >= usize::pow(2, self.depth.try_into().unwrap()) {
            panic!("Tree is full");
        }

        // let hasher = Hash::init(P128Pow5T3, ConstantLength<2>);
        let IncrementalTree { root, zeroes, nodes, depth, position } = self;

        let mut append_leaf = |node, level, index| {
            let level = level as usize;

            if nodes[level].len() > index { nodes[level][index] = node; } 
            else { nodes[level].push(node); }

            if (index % 2) == 1 { 
                IncrementalTree::hash_layer(nodes[level][index - 1], node)
            } else { 
                IncrementalTree::hash_layer(node, zeroes[level])
            }
        };
        
        let mut node = leaf;
        let mut index = *position;
        for level in 0..*depth {
            node = append_leaf(node, level, index);
            index = (index as f64 / 2 as f64).floor() as usize;
        }

        *position += 1;
        *root = node;
        ()
    }

    pub fn witness(&mut self, leaf: Fp) -> (Vec<Fp>, Vec<bool>) {
        let IncrementalTree { zeroes, nodes, depth, .. } = self;

        let index = nodes[0].iter().position(|&el| el == leaf );
        if index.is_none() { panic!("No such leaf"); }

        let mut index = index.unwrap();

        let mut siblings = vec![zeroes[0]; depth.clone()];
        let mut pos = vec![false; depth.clone()];

        let mut sibling_path = |level, index| {
            let level = level as usize;

            if (index % 2) == 1 {
                siblings[level] = nodes[level][index - 1];
                pos[level] = true;
            } else {
                siblings[level] = zeroes[level];
            }
        };

        for level in 0..*depth {
            sibling_path(level, index);
            index = (index as f64 / 2 as f64).floor() as usize;
        }

        (siblings, pos)
    }

    pub fn check_proof(&self, leaf: Fp, siblings: Vec<Fp>, pos: Vec<bool>) -> bool {
        let mut node = leaf;
        for (sibling, p) in siblings.iter().zip(pos.iter()) { 
            if *p {
                node = IncrementalTree::hash_layer(node, *sibling);
            } else {
                node = IncrementalTree::hash_layer(*sibling, node);
            }
        }

        node == self.root
    }

    pub fn root(&self) -> Fp {
        self.root
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    fn hash_layer(left: Fp, right: Fp) -> Fp {
        poseidon::Hash::init(P128Pow5T3, ConstantLength::<2>).hash([left, right])
    }
}


#[cfg(test)]
mod test {

    use super::{IncrementalTree};
    use halo2::pasta::Fp;
    #[test]
    fn construct() {
        let mut tree = IncrementalTree::new(Fp::one(), 3);

        tree.insert(Fp::from(2));
        tree.insert(Fp::from(3));
        tree.insert(Fp::from(4));
        tree.insert(Fp::from(5));
        tree.insert(Fp::from(6));
        tree.insert(Fp::from(7));

        let leaf = Fp::from(7);
        let (siblings, pos) = tree.witness(leaf);
        println!("{:?}", tree.check_proof(leaf, siblings, pos));
    }
}
