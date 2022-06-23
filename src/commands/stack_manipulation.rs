use crate::handler::{Handler, Output, StackObject, output};
use crate::constants;

use rug::{Integer, Float, float::Round};

//clear stack
pub fn clear_stack(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.clear();
    Vec::new()
}

//remove top a objects from stack
pub fn remove_objects(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if let Some(mut num) = int.to_usize() {
                if num>handler.main_stack.len() { num = handler.main_stack.len(); }	//limit clear count
                handler.main_stack.truncate(handler.main_stack.len()-num);
            }
            else {
                output.push(output!(Err, format!("! Cannot possibly remove {} objects from the main stack", int)));
            }
        }
    }
    output
}

//duplicate top of stack
pub fn duplicate_object(handler: &mut Handler) -> Vec<Output> {
    let mut output = Vec::new();
    if handler.main_stack.is_empty() {
        output.push(output!(Err, "! Nothing to duplicate"));
    }
    else {
        handler.main_stack.extend_from_within(handler.main_stack.len()-1..);
    }
    output
}

//duplicate top a objects
pub fn duplicate_objects(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if let Some(num) = int.to_usize() {
                if num<=handler.main_stack.len() {
                    handler.main_stack.extend_from_within(handler.main_stack.len()-num..);
                }
                else {
                    output.push(output!(Err, "! Not enough objects to duplicate"));
                }
            }
            else {
                output.push(output!(Err, format!("! Cannot possibly duplicate {} objects", int)));
            }
        }
    }
    output
}

//swap top 2 objects
pub fn swap_objects(handler: &mut Handler) -> Vec<Output> {
    let mut output = Vec::new();
    if handler.main_stack.len() >= 2 {
        handler.main_stack.swap(handler.main_stack.len()-2, handler.main_stack.len()-1);
    }
    else {
        output.push(output!(Err, "! Not enough objects to rotate"));
    }
    output
}

//rotate top a objects
pub fn rotate_objects(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let mut int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int == 0 as i8 { int = Integer::from(1 as i8); }	//replace 0 with effective no-op
            if let Some(num) = int.clone().abs().to_usize() {
                if num<=handler.main_stack.len() {
                    let sl = handler.main_stack.as_mut_slice();
                    if int < 0 as i8 {
                        sl[handler.main_stack.len()-num..].rotate_left(1);	//if negative, rotate left/down
                    }
                    else {
                        sl[handler.main_stack.len()-num..].rotate_right(1);	//right/up otherwise
                    }
                    handler.main_stack = sl.to_vec();
                }
                else {
                    output.push(output!(Err, "! Not enough objects to rotate"));
                }
            }
            else {
                output.push(output!(Err, format!("! Cannot possibly rotate {} objects", int.abs())));
            }
        }
    }
    output
}

//push stack depth
pub fn push_stack_depth(handler: &mut Handler) -> Vec<Output> {
    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.main_stack.len())));
    Vec::new()
}