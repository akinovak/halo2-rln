use halo2::{
    arithmetic::FieldExt
};
use std::iter;

pub struct IncrementalTree<F: FieldExt> {
    root: F,
    zeroes: Vec<F>,
    nodes: Vec<Vec<F>>,
    depth: usize,
    position: usize
}

impl<F: FieldExt>
IncrementalTree<F> {
    pub fn new(zero_value: F, depth: usize) -> Self {

        if depth > 32 { panic!("MAX DEPTH EXCEEDED") }

        let zeroes: Vec<F> = {
            iter::empty()
            .chain(Some(zero_value))
            .chain(
                (0..depth).scan(zero_value, |zero, _level| {
                    *zero = *zero + *zero;
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
            position: 0
        }
    }

    pub fn append(&mut self, leaf: F) {
        if leaf == self.zeroes[0] {
            panic!("Leaf cannot be equal to zero value");
        }

        if self.nodes.len() >= usize::pow(2, self.depth.try_into().unwrap()) {
            panic!("Tree is full");
        }

        let IncrementalTree { root, zeroes, nodes, depth, position } = self;

        let mut append_leaf = |node, level, index| {
            let level = level as usize;
            let selector = (index % 2) != 0;

            if nodes[level].len() > index {
                nodes[level][index] = node;
            } else {
                nodes[level].push(node);
            }


            let node = match selector {
                true => { 
                    println!("left: {:?} right: {:?}", nodes[level][index - 1], node);
                    nodes[level][index - 1] + node
                },
                false => { 
                    println!("left: {:?} right: {:?}", node, zeroes[level]);
                    node + zeroes[level]
                }
            };


            node
        };

        let mut node = leaf;
        let mut index = *position;
        for level in 0..*depth {
            println!("index: {}", index);
            node = append_leaf(node, level, index);

            index = (index as f64 / 2 as f64).floor() as usize;
        }

        *position += 1;
        *root = node;
        ()
    }

    pub fn root(&self) -> F {
        self.root
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}


#[cfg(test)]
mod test {

    use super::{IncrementalTree};
    use halo2::{
        pasta::Fp
    };
    #[test]
    fn construct() {
        let mut tree = IncrementalTree::<Fp>::new(Fp::one(), 3);

        tree.append(Fp::one() + Fp::one());
        tree.append(Fp::one() + Fp::one());
        tree.append(Fp::one() + Fp::one());
        tree.append(Fp::one() + Fp::one());
        tree.append(Fp::one() + Fp::one());
        tree.append(Fp::one() + Fp::one());


        println!("{:?}", tree.root());
    }
}