use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::{RandGen, RandState}};
use std::io::{stdin, stdout, Write};
use std::time::{SystemTime, Duration};
use std::cmp::Ordering;

mod constants;

//DCIM instance
pub struct Handler<'a> {
    main_stack: Vec<StackObject>, //basic object on a dc stack, can be a number or string

    registers: [Register; 65536],
    register_buffer: RegisterObject,
    direct_register_selector: Option<usize>,


    parameter_stack: (Integer, Integer, Integer), //stores (k,i,o) tuples, used by '{' and '}'
    
    working_precision: u32, //working precision (rug Float mantissa length)

    rng: RandState<'a>, //random number generator
    
}

impl<'a> Default for Handler<'a> {
    fn default() -> Self {
        let working_precision = 256;
        let mut new_rng: RandState<'a> = RandState::new();
        Handler {
            working_precision,
            rng: RandState::new_custom(&mut RandSeed),

            main_stack: Vec::new(),
            registers: [Vec::new(); 65536],
            register_buffer: vec![StackObject::Float(Float::with_val(working_precision, 0 as u32))],
            direct_register_selector: None,
            parameter_stack: (Integer::from(-1 as i32) , Integer::from(10 as i32), Integer::from(10 as i32)),
        }
    }
}


enum StackObject { // : Float + String
    Float(Float),
    String(String),
}
type RegisterObject = Vec<StackObject>; // : Vec<stack_object>
type Register = Vec<RegisterObject>;

struct RandSeed;
impl RandGen for RandSeed {
    fn gen(&mut self) -> u32 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::MAX).as_nanos() as u32 * std::process::id()
    }
}