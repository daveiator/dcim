use crate::handler::{Handler, Output, StackObject, output};
use crate::constants;

use rug::{Float, float::Round};

//set output precision
pub fn set_output_precision(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int>=-1 {
                handler.parameter_stack.last_mut().unwrap().0 = int;
            }
            else {
                output.push(output!(Err, "! Output precision must be at least -1"));
            }
        }
    }
    output
}

//set input base
pub fn set_input_base(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int>=2 {
                handler.parameter_stack.last_mut().unwrap().1 = int;
            }
            else {
                output.push(output!(Err, "! Input base must be at least 2"));
            }
        }
    }
    output
}

//set output base
pub fn set_output_base(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int >= 2 {
                handler.parameter_stack.last_mut().unwrap().2 = int;
            }
            else {
                output.push(output!(Err, "! Output base must be at least 2"));
            }
        }
    }
    output
}

//set working precision
pub fn set_working_precision(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int >= 1 && int <= u32::MAX {
                handler.working_precision = int.to_u32().unwrap();
            }
            else {
                output.push(output!(Err, format!("! Working precision must be between {} and {} (inclusive)", 1, u32::MAX)));
            }
        }
    }
    output
}

//push output precision
pub fn push_output_precision(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().0.clone())));
    Vec::new()
}

//push input base
pub fn push_input_base(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().1.clone())));
    Vec::new()
}

//push output base
pub fn push_output_base(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().2.clone())));
    Vec::new()
}

//push working precision
pub fn push_working_precision(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.working_precision)));
    Vec::new()
}

//create new k,i,o context
pub fn new_context(handler: &mut Handler) -> Vec<Output> {
    handler.parameter_stack.push(Default::default());
    Vec::new()
}

//revert to previous context
pub fn revert_context(handler: &mut Handler) -> Vec<Output> {
    handler.parameter_stack.pop();
    if handler.parameter_stack.is_empty() {
        handler.parameter_stack.push(Default::default());	//ensure 1 entry always remains
    }
    Vec::new()
}