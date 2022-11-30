use std::{ptr::NonNull, slice::from_raw_parts, ops::Not};

mod bindings;

#[derive(Debug)]
pub struct SimpSolver {
    ptr: NonNull<bindings::simp_solver>,
    cex: Vec<Lit>,
}

impl SimpSolver {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::new(unsafe { bindings::simp_solver_new() }).unwrap(),
            cex: Vec::new(),
        }
    }

    pub fn new_var(&mut self) -> Var {
        unsafe { bindings::simp_solver_add_var(self.ptr.as_mut()) }.into()
    }

    pub fn add_clause(&mut self, clause: &[Lit]) {
        unsafe {
            bindings::simp_solver_add_clause(
                self.ptr.as_mut(),
                clause.as_ptr() as *mut _,
                clause.len() as _,
            )
        };
    }

    pub fn solve(&mut self, assumptions: &[Lit]) -> Option<&[Lit]> {
        let ret = unsafe {
            bindings::simp_solver_solve(
                self.ptr.as_ptr(),
                assumptions.as_ptr() as _,
                assumptions.len() as _,
            )
        };
        if ret == 1 {
            let ret = unsafe { bindings::simp_solver_read_cex(self.ptr.as_ptr()) };
            let nvar = unsafe { bindings::simp_solver_nvar(self.ptr.as_mut()) };
            let model = unsafe { from_raw_parts(ret as *const u8, nvar as _) };
            self.cex.clear();
            for var in 0..nvar {
                if model[var as usize] == 2 {
                    continue;
                }
                self.cex
                    .push(Lit::new(var.into(), model[var as usize] == 1));
            }
            Some(&self.cex)
        } else {
            assert!(ret == -1);
            None
        }
    }
}

impl Drop for SimpSolver {
    fn drop(&mut self) {
        unsafe { bindings::simp_solver_delete(self.ptr.as_mut()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Var(i32);

impl From<i32> for Var {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<Var> for i32 {
    fn from(value: Var) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lit(i32);

impl From<Var> for Lit {
    fn from(value: Var) -> Self {
        Self(value.0 + value.0)
    }
}

impl Lit {
    pub fn new(var: Var, compl: bool) -> Self {
        Lit(var.0 + var.0 + compl as i32)
    }

    pub fn var(&self) -> Var {
        Var(self.0 >> 1)
    }

    pub fn compl(&self) -> bool {
        self.0 & 1 > 0
    }
}

impl Not for Lit {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        self.0 ^= 1;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{Lit, SimpSolver};

    #[test]
    fn test() {
        let mut solver = SimpSolver::new();
        let var0 = solver.new_var();
        let var1 = solver.new_var();
        let lit0: Lit = var0.into();
        let lit1: Lit = var1.into();
        solver.add_clause(&[!lit0, lit1]);
        dbg!(solver.solve(&[]));
        solver.add_clause(&[lit0, !lit1]);
        dbg!(solver.solve(&[]));
        solver.add_clause(&[!lit0, !lit1]);
        dbg!(solver.solve(&[]));
        solver.add_clause(&[lit0, lit1]);
        dbg!(solver.solve(&[]));
    }
}
