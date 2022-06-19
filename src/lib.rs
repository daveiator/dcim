use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::RandState};
use std::io::{stdin, stdout, Write};
use std::time::{SystemTime, Duration};
use std::cmp::Ordering;

use crate::constants;

//DCIM instance
pub struct Handler<'a> {
    main_stack: 'a mut Vec<stack_object>, //basic object on a dc stack, can be a number or string

    registers: 'a mut [register; 65536],
    register_buffer: 'a mut register,
    direct_register_selector: 'a mut Option<usize>,


    parameter_stack: 'a mut (Integer, Integer, Integer), //stores (k,i,o) tuples, used by '{' and '}'
    
    working_precision: 'a mut u32, //working precision (rug Float mantissa length)

    rng: RandState, //random number generator
    
}

impl Default for Handler<'a> {
    fn default() -> Self {
        Handler {
            working_precision: 256,
            rng: RandState:new().seed(&(Integer::from(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::MAX).as_nanos()) * std::process::id())),

            main_stack: Vec::new(),
            registers: [Vec::new(); 65536],
            register_buffer: Float::with_val(working_precision, 0),
            parameter_stack: (Integer::from(-1) , Integer::from(10), Integer::from(10)),
        }
    }
}


type stack_object = <T: Float + String>;
type register_object = <T: stack_object + Vec<stack_object>>;
type register = Vec<register_object>;