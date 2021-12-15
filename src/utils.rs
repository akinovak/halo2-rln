use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, Region, AssignedCell},
    plonk::{Column, Advice, Error},
    circuit
};

#[derive(Clone, Debug)]
pub struct NumericCell<F: FieldExt>(AssignedCell<F, F>);

pub trait Numeric<F: FieldExt>: Clone + std::fmt::Debug {
    fn new(assigned: AssignedCell<F, F>) -> Self;
    fn value(&self) -> Option<F>;
    fn cell(&self) -> circuit::Cell;
}

impl<F: FieldExt> Numeric<F> for NumericCell<F> {
    fn new(assigned: AssignedCell<F, F>) -> Self {
        NumericCell(assigned)
    }
    fn value(&self) -> Option<F> {
        self.0.value().map(|val| *val)
    }

    fn cell(&self) -> circuit::Cell {
        self.0.cell()
    }
}

impl<F: FieldExt> NumericCell<F>
where
{
    pub fn copy<A, AR>(
        &self,
        annotation: A,
        region: &mut Region<'_, F>,
        column: Column<Advice>,
        offset: usize,
    ) -> Result<Self, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let assigned_cell = &self.0;
        let copied = assigned_cell.copy_advice(annotation, region, column, offset)?;
        Ok(Numeric::new(copied))
    }
}


pub trait UtilitiesInstructions<F: FieldExt> {
    type Var: Numeric<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        column: Column<Advice>,
        value: Option<F>,
    ) -> Result<Self::Var, Error> {
        layouter.assign_region(
            || "load private",
            |mut region| {
                let assigned = region.assign_advice(
                    || "load private",
                    column,
                    0,
                    || value.ok_or(Error::Synthesis),
                )?;
                Ok(Numeric::new(assigned))
            },
        )
    }
}