use rug::{Integer, Float, rand::RandState};

use std::sync::Arc;

use crate::commands;

use lazy_static::lazy_static;
lazy_static! {
    static ref RNG: Arc<RandState<'static>> = Arc::new(RandState::new());
}

//DCIM instance
pub struct Handler {
    main_stack: Vec<StackObject>, //basic object on a dc stack, can be a number or string

    registers: Vec<Register>,
    register_buffer: RegisterObject,
    direct_register_selector: Option<usize>,

    parameter_stack: Vec<(Integer, Integer, Integer)>, //stores (k,i,o) tuples, used by '{' and '}'
    
    working_precision: u32, //working precision (rug Float mantissa length)

}

impl Default for Handler {
    fn default() -> Self {
        let working_precision = 256;
        Handler {
            working_precision,

            main_stack: Vec::new(),
            registers: vec![Vec::new(); 65536],
            register_buffer: vec![StackObject::Float(Float::with_val(working_precision, 0 as u32))],
            direct_register_selector: None,
            parameter_stack: vec![(Integer::from(-1 as i32) , Integer::from(10 as i32), Integer::from(10 as i32))],
        }
    }
}

impl Handler {
    pub fn new(working_precision: u32) -> Self {
        Handler { working_precision, ..Default::default() }
    }

    pub fn handle(&mut self, input: Input) -> Vec<Output> {
        let mut output = Vec::new();
        match input {
            Input::Interactive(input) => {
                commands::execute(self, input.to_string()).into_iter().for_each(|x| output.push(x));
                return output;
            },
            Input::Expression(expressions) => {
                if expressions.is_empty() {
                    output.push(Err("! Empty expression".to_string()));
                    return output;
                }
                for i in 0..expressions.len() {
                    if i==expressions.len()-1 && expressions[i]=="?" {
                        //if last expression is "?", request interactive mode
                        output.push(Ok((Some("File read!".to_string()), Command::Interactive)));
                        return output;
                    }
                    commands::execute(self, expressions[i].to_string()).into_iter().for_each(|x| output.push(x));
                    return output;
                }
            },
            Input::File(files) => {
                if files.is_empty() {
                     output.push(Err("! No file name provided".to_string()));
                     return output;
                } else {
                    for i in 0..files.len() {
                        if i==files.len()-1 && files[i]=="?" {
                            //if last filename is "?", request interactive mode
                            output.push(Ok((Some("File read!".to_string()), Command::Interactive)));
                            return output;
                        }
                        match std::fs::read_to_string(files[i]) {
                            Ok(content) => {
                                let no_comments = content.lines().map(|line| line.split_once('#').unwrap_or((line,"")).0).collect::<Vec<&str>>().join("\n");
                                commands::execute(self, no_comments).into_iter().for_each(|x| output.push(x));
                                return output;
                            },
                            Err(error) => {
                                output.push(Err(format!("! Unable to read file \"{}\": {}", files[i], error)));
                                return output;
                            },
                        }
                    }
                }
            },
        }
        vec!(Err("! Didn't get a valid input (Somehow)".to_string()))
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Self {
            working_precision: self.working_precision,
            main_stack: self.main_stack.clone(),
            registers: self.registers.clone(),
            register_buffer: self.register_buffer.clone(),
            direct_register_selector: self.direct_register_selector,
            parameter_stack: self.parameter_stack.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.working_precision = source.working_precision;
        self.main_stack = source.main_stack.clone();
        self.registers = source.registers.clone();
        self.register_buffer = source.register_buffer.clone();
        self.direct_register_selector = source.direct_register_selector;
        self.parameter_stack = source.parameter_stack.clone();
    }
}

#[derive(Clone, Debug)]
enum StackObject { // : Float + String
    Float(Float),
    String(String),
}

type RegisterObject = Vec<StackObject>; // : Vec<stack_object>
type Register = Vec<RegisterObject>;

pub type Output<'a> = Result<(Option<String>, Command), String>;

pub enum Command {
    Exit,
    Restart,
    Interactive,
    None,
}

pub enum Input<'a> {
    Interactive(&'a str),
    Expression(Vec<&'a str>),
    File(Vec<&'a str>),
}