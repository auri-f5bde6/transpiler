use crate::lmc::{BRA, BRP, BRZ, LDA, Program, ProgramLine, STA};

pub struct Optimiser {
    index: usize,
    program: Program,
}
impl Optimiser {
    pub fn new(program: Program) -> Optimiser {
        Optimiser { program, index: 0 }
    }
    fn peek(&self, offset: usize) -> Option<&ProgramLine> {
        if (self.index + offset > self.program.0.len()) {
            return None;
        }
        Some(&self.program[self.index + offset])
    }
    fn current(&self) -> Option<&ProgramLine> {
        self.peek(0)
    }
    fn set_current(&mut self, program_line: ProgramLine) {
        self.program.0[self.index] = program_line;
    }
    pub fn optimise(mut self) -> Program {
        while self.index < self.program.0.len() - 1 {
            self.a();
            self.b();
            self.index += 1;
        }
        self.program
    }

    // Remove the lda x call in
    // sta x
    // lda x
    fn a(&mut self) -> Option<()> {
        let current = self.current()?;
        let next = self.peek(1)?;
        if (current.is(STA)
            && next.is(LDA)
            && current.get_operand() == next.get_operand()
            && current.label.is_none()
            && next.label.is_none())
        {
            self.program.remove(self.index + 1);
            self.index -= 1;
        }
        Some(())
    }

    fn snapshot_index<T: Fn(&mut Optimiser) -> ()>(&mut self, func: T) {
        let index_snapshot = self.index;
        (func)(self);
        self.index = index_snapshot;
    }

    // Remove the useless BRA
    //         BR? label_1
    //         ...
    // label_1 BRA label_2
    // label_2 ...
    fn b(&mut self) -> Option<()> {
        let current = self.current()?;
        let next = self.peek(1)?;
        if (current.is(BRA)
            && current.label.is_some()
            && current
            .get_operand()
            .is_some_and(|label_2| label_2 == next.label.as_ref().unwrap()))
        {
            let label_1 = current.label.to_owned().unwrap();
            let label_2 = current.inst.get_operand().map(String::from).unwrap();
            let merged = format!("merged_{}_{}", label_1, label_2);
            self.program.remove(self.index);
            self.snapshot_index(|optimiser| {
                while optimiser.index > 0 {
                    let current = optimiser.current().unwrap();
                    if (current
                        .get_operand()
                        .is_some_and(|operand| operand == label_1 || operand == label_2))
                    {
                        optimiser.set_current(ProgramLine::new(
                            current.label.clone(),
                            current.inst.with_operand(&merged),
                        ))
                    }
                    optimiser.index -= 1;
                }
            });

            // Update the label_2 into the merged label (the BRA is removed and the next instruction is shifted up, so same index)
            let current = self.current().unwrap();
            self.set_current(ProgramLine::new(Some(merged), current.inst.clone_boxed()));
            return Some(());
        }
        None
    }
    /*
    Remove useless instruction for while loop with comparison (or any derivative i.e. multiplication)
                         BRZ z (optional)
                         BRP p
                       z LDA literal_1
                         BRA label_continue_true
                       p LDA literal_0
     label_continue_true BRZ label_continue_false
                         ...
    label_continue_false ...

                         BRZ z (optional)
                         BRP p
                       z BRA label_continue_true
                       p BRA label_continue_false
     label_continue_true ...
    label_continue_false ...
    */
    /*fn c(&mut self) -> Option<()> {
        let mut on_zero = None;
        let optional = self.current()?;
        if (optional.is(BRZ)) {
            on_zero = Some(optional.get_operand());
            self.index += 1;
        }
        let first = self.current()?;
        let second = self.peek(1)?;
        let third = self.peek(2)?;
        let forth = self.peek(3)?;
        let fifth = self.peek(4)?;
        if (first.is(BRP)
            && second.is(LDA)
            && third.is(BRA)
            && forth.is(LDA)
            && fifth.is(BRZ)
            && second.get_operand().is_some_and(|s| s == "literal_1")
            && forth.get_operand().is_some_and(|s| s == "literal_0"))
        {}
    }*/
}
