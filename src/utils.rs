use crate::halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, Region, AssignedCell},
    plonk::{Column, Advice, Error, Instance},
    circuit
};

#[derive(Clone, Debug)]
pub struct NumericCell<F: FieldExt>(AssignedCell<F, F>);

pub trait Numeric<F: FieldExt>: Clone + std::fmt::Debug {
    fn new(assigned: AssignedCell<F, F>) -> Self;
    fn value(&self) -> Option<F>;
    fn cell(&self) -> circuit::Cell;
    fn to_cell_val(&self) -> CellValue<F>;
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

    fn to_cell_val(&self) -> CellValue<F> {
        CellValue::new(self.cell(), self.value())
    }
}

impl<F: FieldExt> From<NumericCell<F>> for CellValue<F> {
    fn from(cell: NumericCell<F>) -> CellValue<F> {
        CellValue::new(cell.cell(), cell.value())
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

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        column: Column<Instance>,
        var: Self::Var,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(var.cell(), column, row)
    }
}

//tmp hack until assigned cell is not fixed to work with poseidon api
pub fn from_cell_vale_to_numeric<F: FieldExt>(
    mut layouter: impl Layouter<F>,
    column: Column<Advice>,
    value: Option<F>,
) -> Result<NumericCell<F>, Error>
{
    let assigned = layouter.assign_region(
        || "just assign",
        |mut region| {
            let assigned = region.assign_advice(
                || "assigned",
                column,
                0,
                || value.ok_or(Error::Synthesis),
            )?;
            Ok(Numeric::new(assigned))
        },
    )?;

    Ok(assigned)
}

#[derive(Copy, Clone, Debug)]
pub struct CellValue<F: FieldExt> {
    pub cell: circuit::Cell,
    pub value: Option<F>,
}

pub trait Var<F: FieldExt>: Copy + Clone + std::fmt::Debug {
    fn new(cell: circuit::Cell, value: Option<F>) -> Self;
    fn cell(&self) -> circuit::Cell;
    fn value(&self) -> Option<F>;
}

impl<F: FieldExt> Var<F> for CellValue<F> {
    fn new(cell: circuit::Cell, value: Option<F>) -> Self {
        Self { cell, value }
    }

    fn cell(&self) -> circuit::Cell {
        self.cell
    }

    fn value(&self) -> Option<F> {
        self.value
    }
}