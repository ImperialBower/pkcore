//! Three shells, one kernel — SPIKE.
//!
//! Proves the claim that motivated the extraction: `&mut self` (Table),
//! `&self` + interior mutability (TableCelled), and value semantics
//! (TableImmutable) are *drivers* of the same pure transition functions, not
//! three engines. Each shell below is a delegation wrapper with zero betting
//! logic of its own. A defect fixed in `kernel.rs` is fixed in all three by
//! construction — the property DEFECT_015 proved the sibling pair lacks.

use crate::kernel::{self, Event, HandBetting, KError, Step};
use std::cell::RefCell;

/// The `Table` discipline: conventional `&mut self`.
pub struct TableMut {
    pub state: HandBetting,
    pub event_log: Vec<Event>,
}

impl TableMut {
    pub fn new(state: HandBetting) -> Self {
        TableMut { state, event_log: Vec::new() }
    }
    pub fn act_raise(&mut self, seat: u8, amount: usize) -> Result<usize, KError> {
        let step = kernel::act_raise(&self.state, seat, amount)?;
        self.apply(step)
    }
    pub fn act_call(&mut self, seat: u8) -> Result<usize, KError> {
        let step = kernel::act_call(&self.state, seat)?;
        self.apply(step)
    }
    pub fn act_all_in(&mut self, seat: u8) -> Result<usize, KError> {
        let step = kernel::act_all_in(&self.state, seat)?;
        self.apply(step)
    }
    fn apply(&mut self, step: Step) -> Result<usize, KError> {
        self.state = step.next;
        self.event_log.extend(step.events);
        Ok(step.returned)
    }
}

/// The `TableCelled` discipline: every method takes `&self`.
/// One `RefCell` around the whole value replaces the per-field
/// `Cell`/`RefCell` lattice (`ANALYSIS_TableCelled_vs_Table.md`) — interior
/// mutability survives as a *shell property* instead of a state design.
pub struct TableCelledLite {
    inner: RefCell<(HandBetting, Vec<Event>)>,
}

impl TableCelledLite {
    pub fn new(state: HandBetting) -> Self {
        TableCelledLite { inner: RefCell::new((state, Vec::new())) }
    }
    pub fn act_raise(&self, seat: u8, amount: usize) -> Result<usize, KError> {
        let step = kernel::act_raise(&self.inner.borrow().0, seat, amount)?;
        self.apply(step)
    }
    pub fn act_call(&self, seat: u8) -> Result<usize, KError> {
        let step = kernel::act_call(&self.inner.borrow().0, seat)?;
        self.apply(step)
    }
    pub fn act_all_in(&self, seat: u8) -> Result<usize, KError> {
        let step = kernel::act_all_in(&self.inner.borrow().0, seat)?;
        self.apply(step)
    }
    fn apply(&self, step: Step) -> Result<usize, KError> {
        let mut guard = self.inner.borrow_mut();
        guard.0 = step.next;
        guard.1.extend(step.events);
        Ok(step.returned)
    }
    pub fn snapshot(&self) -> (HandBetting, Vec<Event>) {
        self.inner.borrow().clone()
    }
}

/// The immutable discipline needs no wrapper at all: the kernel *is*
/// `TableImmutable`. Provided as a free function for symmetry in the
/// equivalence test.
pub fn immutable_act_raise(state: &HandBetting, seat: u8, amount: usize) -> Result<Step, KError> {
    kernel::act_raise(state, seat, amount)
}
