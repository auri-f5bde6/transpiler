use paste::paste;
use std::concat;

pub struct Program(Vec<(Option<String>, Box<dyn Instruction>)>);

impl Program {
    pub fn new() -> Program {
        Program(Vec::new())
    }
    pub fn push(&mut self, label: Option<String>, instruction: Box<dyn Instruction>) {
        self.0.push((label, instruction));
    }
    pub fn get_program(&self) -> String {
        let mut longest_label_len = 0;
        for (label, _) in self.0.iter() {
            let len = label.as_ref().map(|s| s.len()).unwrap_or(0);
            if len > longest_label_len {
                longest_label_len = len
            }
        }

        let mut result = String::new();
        for (label, inst) in self.0.iter() {
            if let Some(label) = label {
                result.push_str(&" ".repeat(longest_label_len - label.len()));
                result.push_str(label);
                result.push(' ');
            } else {
                result.push_str(&" ".repeat(longest_label_len + 1));
            }
            result.push_str(inst.get_mnemonic().as_str());
            result.push('\n');
        }
        result
    }
    pub fn merge(self, other: Program) -> Program {
        Program(self.0.into_iter().chain(other.0).collect())
    }
    fn get_instruction(&self, index: usize) -> &(Option<String>, Box<dyn Instruction>) {
        &self.0[index]
    }
    pub fn optimise(&mut self) {
        let mut pos = 0;
        while pos < self.0.len() - 1 {
            let a = self.get_instruction(pos);
            let b = self.get_instruction(pos + 1);

            // Remove useless calls, e.g.

            // Remove the lda x call
            // sta x
            // lda x
            if (a.1.get_numeric() == 300
                && b.1.get_numeric() == 500
                && a.1.get_operand() == b.1.get_operand()
                && a.0.is_none()
                && b.0.is_none())
            {
                self.0.remove(pos + 1);
                pos -= 1
            }

            pos += 1;
        }
    }

    pub fn push_dat(&mut self, label: Option<String>, dat: u16) {
        self.push(label, Dat::new(dat));
    }
}

pub trait Instruction {
    fn get_operand(&self) -> Option<&str>;
    fn get_numeric(&self) -> u16;
    fn get_mnemonic(&self) -> String;
}

macro_rules! no_operand {
    ($name:ident, $numeric: literal) => {
        paste! {
            pub struct [<$name:camel>];
            impl [<$name:camel>]{
                pub fn new() -> Box<Self>{
                    Box::new(Self{})
                }
            }
            impl Instruction for [<$name:camel>] {
                fn get_operand(&self) -> Option<&str> {
                    None
                }
                fn get_numeric(&self) -> u16 {
                    $numeric
                }
                fn get_mnemonic(&self) -> String {
                    // Maybe I should make the return type a enum variant of String and &'static str? Feels a bit like premature optimisation
                   String::from(stringify!($name))
                }
            }
        }
    };
}

macro_rules! with_operand {
    ($name:ident, $numeric: literal) => {
        paste! {
            pub struct [<$name:camel>]{
                operand: String
            }
            impl [<$name:camel>]{
                pub fn new(operand: String) -> Box<Self>{
                    Box::new(Self{operand})
                }
            }
            impl Instruction for [<$name:camel>] {
                fn get_operand(&self) -> Option<&str> {
                    Some(&self.operand)
                }
                fn get_numeric(&self) -> u16 {
                    $numeric
                }
                fn get_mnemonic(&self) -> String {
                    format!(concat!(stringify!($name), " {}"), self.operand)
                }
            }
        }
    };
}

with_operand!(ADD, 100);
with_operand!(SUB, 200);
with_operand!(STA, 300);
with_operand!(LDA, 500);
with_operand!(BRA, 600);
with_operand!(BRZ, 700);
with_operand!(BRP, 800);
no_operand!(INP, 901);
no_operand!(OUT, 902);
no_operand!(HLT, 0);

struct Dat {
    operand: String,
    value: u16,
}
impl Dat {
    fn new(operand: u16) -> Box<Self> {
        Box::new(Self {
            operand: operand.to_string(),
            value: operand,
        })
    }
}
impl Instruction for Dat {
    fn get_operand(&self) -> Option<&str> {
        return Some(&self.operand);
    }
    fn get_numeric(&self) -> u16 {
        self.value
    }

    fn get_mnemonic(&self) -> String {
        format!("DAT {}", self.operand)
    }
}
