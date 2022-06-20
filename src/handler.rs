use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::{RandGen, RandState}};

use std::cmp::Ordering;
use std::sync::Arc;
use std::convert::TryInto;
use std::fs::File;

use crate::constants;

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
    /*
    pub fn new_with_mode(working_precision: u32, mode: SessionMode) -> Self {
        Handler { working_precision, mode ..Default::default() }
    }*/
    pub fn handle(&mut self, input: Input) -> Vec<Output> {
        vec!(Ok(("Ok", Command::None)))
    }
}

#[derive(Clone, Debug)]
enum StackObject { // : Float + String
    Float(Float),
    String(String),
}

type RegisterObject = Vec<StackObject>; // : Vec<stack_object>
type Register = Vec<RegisterObject>;

pub type Output<'a> = Result<(&'a str, Command), &'a str>;

pub enum Command {
    Exit,
    Restart,
    None,
}

pub enum Input<'a> {
    Interactive(&'a str),
    Expression(Vec<&'a str>),
    File(File),
    Help,
}