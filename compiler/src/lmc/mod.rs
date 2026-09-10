use paste::paste;
use std::concat;
use std::ops::Index;

pub struct ProgramLine {
    pub(crate) label: Option<String>,
    pub(crate) inst: Box<dyn Instruction>,
}
impl ProgramLine {
    pub fn new(label: Option<String>, inst: Box<dyn Instruction>) -> ProgramLine {
        ProgramLine { label, inst }
    }
    pub fn is(&self, other: u16) -> bool {
        self.inst.is(other)
    }
    pub fn get_operand(&self) -> Option<&str> {
        self.inst.get_operand()
    }
}
pub struct Program(pub(crate) Vec<ProgramLine>);
impl Index<usize> for Program {
    type Output = ProgramLine;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
impl Program {
    pub fn new() -> Program {
        Program(Vec::new())
    }
    pub fn push(&mut self, label: Option<String>, instruction: Box<dyn Instruction>) {
        self.0.push(ProgramLine::new(label, instruction));
    }
    pub fn remove(&mut self, index: usize) -> ProgramLine {
        self.0.remove(index)
    }
    pub fn get_program(&self) -> String {
        let mut longest_label_len = 0;
        for ProgramLine { label, inst: _inst } in self.0.iter() {
            let len = label.as_ref().map(|s| s.len()).unwrap_or(0);
            if len > longest_label_len {
                longest_label_len = len
            }
        }

        let mut result = String::new();
        for ProgramLine { label, inst } in self.0.iter() {
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
    fn get_instruction(&self, index: usize) -> &ProgramLine {
        &self[index]
    }

    pub fn push_dat(&mut self, label: Option<String>, dat: u16) {
        self.push(label, Dat::new(dat));
    }
}

pub trait Instruction {
    fn get_operand(&self) -> Option<&str>;
    fn get_numeric(&self) -> u16;
    fn is(&self, other: u16) -> bool;
    fn get_mnemonic(&self) -> String;
    fn with_operand(&self, operand: &str) -> Box<dyn Instruction>;
    fn clone_boxed(&self) -> Box<dyn Instruction>;
}


macro_rules! no_operand {
    ($name:ident) => {
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
                    $name
                }
                fn is(&self, other: u16)->bool{
                    $name == other
                }
                fn get_mnemonic(&self) -> String {
                    // Maybe I should make the return type a enum variant of String and &'static str? Feels a bit like premature optimisation
                   String::from(stringify!($name))
                }
                fn with_operand(&self, operand: &str) -> Box<dyn Instruction>{
                    let _ = operand;
                    panic!(concat!(stringify!($name), " do not take any operand"))
                }
                fn clone_boxed(&self)->Box<dyn  Instruction>{
                    return Self::new();
                }
            }
        }
    };
}

macro_rules! with_operand {
    ($name:ident) => {
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
                    $name
                }
                fn is(&self, other: u16)->bool{
                    $name == other
                }
                fn get_mnemonic(&self) -> String {
                    format!(concat!(stringify!($name), " {}"), self.operand)
                }
                fn with_operand(&self, operand: &str) -> Box<dyn Instruction>{
                    return Self::new(String::from(operand))
                }
                fn clone_boxed(&self)->Box<dyn Instruction>{
                    return Self::new(self.operand.clone());
                }
            }
        }
    };
}
pub const ADD: u16 = 100;
pub const SUB: u16 = 200;
pub const STA: u16 = 300;
pub const LDA: u16 = 500;
pub const BRA: u16 = 600;
pub const BRZ: u16 = 700;
pub const BRP: u16 = 800;
pub const INP: u16 = 901;
pub const OUT: u16 = 902;
pub const HLT: u16 = 0;

with_operand!(ADD);
with_operand!(SUB);
with_operand!(STA);
with_operand!(LDA);
with_operand!(BRA);
with_operand!(BRZ);
with_operand!(BRP);
no_operand!(INP);
no_operand!(OUT);
no_operand!(HLT);

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
    fn is(&self, other: u16) -> bool {
        self.value == other
    }
    fn get_mnemonic(&self) -> String {
        format!("DAT {}", self.operand)
    }

    fn with_operand(&self, operand: &str) -> Box<dyn Instruction> {
        Self::new(operand.parse().expect("operand is not a valid number"))
    }

    fn clone_boxed(&self) -> Box<dyn Instruction> {
        Self::new(self.value)
    }
}
