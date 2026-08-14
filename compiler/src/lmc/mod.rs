use std::concat;
use paste::paste;

pub struct Program(Vec<Box<dyn Instruction>>);

impl Program {
    pub fn new() -> Program {
        Program(Vec::new())
    }
    pub fn push(&mut self, instruction: Box<dyn Instruction>) {
        self.0.push(instruction);
    }
}

pub trait Instruction {
    fn new(mailbox: Option<u16>) -> Box<Self>
    where
        Self: Sized;
    fn get_numeric(&self) -> u16;
    fn get_mnemonic(&self) -> String;
}

macro_rules! lone_instruction {
    ($name:ident, $numeric: literal) => {
        paste!{
            pub struct [<$name:camel>];
            impl Instruction for [<$name:camel>] {
                fn new(mailbox: Option<u16>) -> Box<Self>{
                    if !mailbox.is_none() {
                        let panic_message = concat!(stringify!($name), " do not mailbox to be passed as argument, should be None");
                        panic!("{}", panic_message);
                    }
                    Box::new(Self{})
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

macro_rules! instruction_take_mailbox_as_reference {
    ($name:ident, $numeric: literal) => {
        paste!{
            pub struct [<$name:camel>]{
                mailbox: u16
            }
            impl Instruction for [<$name:camel>] {
                fn new(mailbox: Option<u16>) -> Box<Self>{
                    Box::new(Self{mailbox: mailbox.expect(concat!(stringify!($name), " expect mailbox to be passed as argument"))})
                }
                fn get_numeric(&self) -> u16 {
                    $numeric + self.mailbox
                }
                fn get_mnemonic(&self) -> String {
                    format!(concat!(stringify!($name), " {}"), self.mailbox)
                }
            }
        }
    };
}

instruction_take_mailbox_as_reference!(ADD, 100);
instruction_take_mailbox_as_reference!(SUB, 200);
instruction_take_mailbox_as_reference!(STA, 300);
instruction_take_mailbox_as_reference!(LDA, 500);
instruction_take_mailbox_as_reference!(BRA, 600);
instruction_take_mailbox_as_reference!(BRZ, 700);
instruction_take_mailbox_as_reference!(BRP, 800);
lone_instruction!(INP, 901);
lone_instruction!(OUT, 902);
lone_instruction!(HLT, 0);

struct Dat {
    data: u16,
}
impl Instruction for Dat {
    fn new(mailbox: Option<u16>) -> Box<Self> {
        Box::new(Self { data: mailbox.unwrap_or(0) })
    }
    fn get_numeric(&self) -> u16 {
        self.data
    }

    fn get_mnemonic(&self) -> String {
        format!("DAT {}", self.data)
    }
}