use crate::handler::{Handler, Output, StackObject, Command, output};
use crate::constants;

use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::RandState};

use std::sync::Arc;

//add or concatenate strings
pub fn add(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            //concat strings or add floats
            handler.main_stack.push(a + b);
        }
    }
    output
}

//subtract or remove chars from string
pub fn subtract(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            //remove b chars from string a
            if a.is_string() {
                let mut newstr = a.get_string().chars().collect::<Vec<char>>();
                let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;	//extract b, keep for checking if negative
                if let Some(mut num) = &int.abs_ref().complete().to_usize() {
                    if num>newstr.len() { num = newstr.len(); }	//account for too large b
                    if int < (0 as i8) { newstr.reverse(); }	//if negative, reverse to remove from front
                    newstr.truncate(newstr.len()-num);
                    if int < (0 as i8) { newstr.reverse(); }	//undo reversal
                    handler.main_stack.push(StackObject::String(newstr.iter().collect::<String>()));
                }
                else {
                    output.push(output!(Err, format!("! Cannot possibly remove {} characters from a string", int)));
                }
            }
            //subtract numbers
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float() - b.get_float())));
            }
        }
    }
    output
}

//multiply or repeat/invert string
pub fn multiply(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            //repeat string a b times
            if a.is_string() {
                let mut newstr = *a.get_string();
                let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;	//extract b, keep for checking if negative
                if let Some(mut num) = &int.abs_ref().complete().to_usize() {
                    if num*newstr.len()>usize::MAX { num = usize::MAX/newstr.len(); }	//account for too large b
                    newstr = newstr.repeat(num);
                    if int < (0 as i8) { newstr = super::rev_str(newstr); }	//if b is negative, invert string
                    handler.main_stack.push(StackObject::String(newstr));
                }
                else {
                    output.push(output!(Err, format!("! Cannot possibly repeat a string {} times", int)));
                }
            }
            //multiply numbers
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float() * b.get_float())));
            }
        }
    }
    output
}

//divide or shorten string to length
pub fn divide(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            //shorten string a to length b
            if a.is_string() {
                let mut newstr = a.get_string().chars().collect::<Vec<char>>();
                let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;	//extract b, keep for checking if negative
                if let Some(num) = &int.abs_ref().complete().to_usize() {
                    if int < (0 as i8) { newstr.reverse(); }	//if negative, reverse to remove from front
                    newstr.truncate(*num);
                    if int < (0 as i8) { newstr.reverse(); }	//undo reversal
                    handler.main_stack.push(StackObject::String(newstr.iter().collect::<String>()));
                }
                else {
                    output.push(output!(Err, format!("! Cannot possibly shorten a string to {} characters", int)));
                }
            }
            //divide numbers
            else {
                if *b.get_float() == 0 {
                    output.push(output!(Err, "! Arithmetic error: Attempted division by zero"));
                }
                else {
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float() / b.get_float())));
                }
            }
        }
    }
    output
}

//modulo, integers only
pub fn modulo(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            let ia = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            let ib = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if ib==0 {
                output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
            }
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, ia % ib)));
            }
        }
    }
    output
}

//euclidean division or split string
pub fn euclidean_division(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            if a.is_string() {
                let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
                if let Some(mut idx) = &int.to_usize() {
                    let cvec = a.get_string().chars().collect::<Vec<char>>();
                    if idx>cvec.len() { idx=cvec.len(); }	//if too large, split at max index to preserve signature
                    handler.main_stack.push(StackObject::String(cvec[0..idx].iter().collect::<String>()));
                    handler.main_stack.push(StackObject::String(cvec[idx..].iter().collect::<String>()));
                }
                else {
                    output.push(output!(Err, format!("! Cannot possibly split a string at character {}", int)));
                }
            }
            else {
                let ia = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
                let ib = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
                if ib == 0 as i8 {
                    output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
                }
                else {
                    let (quot, rem)=ia.div_rem_euc(ib);
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, quot)));
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, rem)));
                }
            }
        }
    }
    output
}

//exponentiation
pub fn exponent(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            if *a.get_float() < 0 as i8 && b.get_float().clone().abs() < 1 as i8{
                output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
            }
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().pow(b.get_float()))));
            }
        }
    }
    output
}

//modular exponentiation, integers only
pub fn modular_exponent(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let c = handler.main_stack.pop().unwrap();
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), Some(&c), &mut output) {
            let ia = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            let ib = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            let ic = c.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if ic == 0 as i8 {
                output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
            }
            else {
                if let Ok(res) = ia.clone().pow_mod(&ib, &ic) {
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, res)));
                }
                else {
                    output.push(output!(Err, format!("! Arithmetic error: {} doesn't have an inverse mod {}", ia, ic)));
                }
            }
        }
    }
    output
}

//square root
pub fn square_root(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if *a.get_float() < 0 {
                output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
            }
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().sqrt())));
            }
        }
    }
    output
}

//nth root (bth)
pub fn nth_root(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            if *a.get_float() < 0 as i8 && b.get_float().clone().abs() > 1 as i8 {
                output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
            }
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().pow(b.get_float().recip()))));
            }
        }
    }
    output
}

//length of string or natural logarithm
pub fn ln(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.is_string() {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_string().chars().count())));
            }
            else {
                if *a.get_float() <= 0 {
                    output.push(output!(Err, "! Arithmetic error: Logarithms of zero and negative numbers are not allowed"));
                }
                else {
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().ln())));
                }
            }
        }
    }
    output
}

//base b logarithm
pub fn log(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            if *a.get_float() <= 0 as i8 {
                output.push(output!(Err, "! Arithmetic error: Logarithms of zero and negative numbers are not allowed"));
            }
            else if *b.get_float() == 1 as i8 || *b.get_float() <= 0 as i8 {
                output.push(output!(Err, "! Arithmetic error: Logarithm base must be positive and not equal to 1"));
            }
            else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().ln()/b.get_float().ln())));
            }
        }
    }
    output
}

//sine
pub fn sin(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().sin())));
        }
    }
    output
}

//cosine
pub fn cos(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().cos())));
        }
    }
    output
}

//tangent
pub fn tan(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().tan())));
        }
    }
    output
}

//arc-sine
pub fn asin(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.get_float().clone().abs() > 1 as i8 {
                output.push(output!(Err, "! Arithmetic error: Arc-sine of value outside [-1,1]"));
            } else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().asin())));
            }
        }
    }
    output
}

//arc-cosine
pub fn acos(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.get_float().clone().abs() > 1 as i8 {
                output.push(output!(Err, "! Arithmetic error: Arc-cosine of value outside [-1,1]"));
            } else {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().acos())));
            }
        }
    }
    output
}

//arc-tangent
pub fn atan(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_float().atan())));
        }
    }
    output
}

//random integer [0;a)
pub fn random_int(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output){
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if int <= 0 as i8 {
                output.push(output!(Err, "! Upper bound for random value must be above 0"));
            }
            else {
                // handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, int.random_below()))); // TODO FIX RAND *(Arc::(clone(constants::env::RNG))))));
            }
        }
    }
    output
}