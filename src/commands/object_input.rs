use crate::handler::{Handler, Output, StackObject, output};

use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::RandState};


//standard number input, force with single quote to use letters
pub fn standard_number_input<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if handler.parameter_stack.last().unwrap().1>(36 as i32) {
        output.push(output!(Err, "! Any-base input must be used for input bases over 36"));
    }
    else {
        let mut numstr = String::new();	//gets filled with number to be parsed later
        let mut frac = false;	//'.' has already occurred
        let mut neg = false;	//'_' has already occurred
        let mut alpha = false;	//letters are used
        if command == '\'' {
            alpha = true;
            command = command_stack.last_mut().unwrap().pop().unwrap_or('\0');
        }
        //keep adding to numstr until number is finished
        'STDNUM_FINISHED: loop {
            //numbers, periods and exponential notation
            if command.is_ascii_digit()||command == '.'||command == '@' {
                if command == '.' { if frac { break 'STDNUM_FINISHED; } else { frac = true; } } //break on encountering second '.'
                if command == '@' { neg = false; }	//allow for second negative sign in exponent
                numstr.push(command);						
            }
            //'_' needs to be replaced with '-'
            else if command == '_' {
                if neg { break 'STDNUM_FINISHED; } else { neg = true; } //break on encountering second '_'
                numstr.push('-');
            }
            //parse letters if number is prefixed with quote
            else if command.is_ascii_alphabetic() {
                if alpha {
                    numstr.push(command);							
                }
                else {
                    break 'STDNUM_FINISHED;
                }
            }
            else {
                break 'STDNUM_FINISHED;
            }
            command = command_stack.last_mut().unwrap().pop().unwrap_or('\0');
        }
        command_stack.last_mut().unwrap().push(command);	//restore first char that isn't part of the number
        if numstr.starts_with('@') { numstr.insert(0, '1') }	//add implied 1 before exponential marker
        if numstr.starts_with('.')||numstr.starts_with("-.") { numstr = numstr.replace('.', "0."); }	//add implied zero before fractional separator
        if numstr.ends_with('.')||numstr.ends_with('-')||numstr.is_empty() { numstr.push('0'); }	//add implied zero at end
        match Float::parse_radix(numstr.clone(), handler.parameter_stack.last().unwrap().1.to_i32().unwrap()) {		
            Ok(res) => {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, res)));
            },
            Err(error) => {
                output.push(output!(Err, format!("! Unable to parse number \"{}\": {}", numstr, error)));
            },
        }
    }
    output
}

//any-base number input
pub fn any_base_number_input<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    let mut num = Integer::from(0 as i16);	//resulting number
    if command_stack.last().unwrap().is_empty() {
        handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, num)));	//default to 0 if on end of input
    }
    else {
        let ibase = handler.parameter_stack.last().unwrap().1.clone();
        let mut dig = String::new();	//digit being parsed
        let mut neg = false;	//number negative?
        let mut frac = false;	//fractional separator has occurred
        let mut scale = Integer::from(1 as i16);	//scale to divide by, for non-integers
        let mut exp = false;	//exponential symbol has occurred
        'CANCEL_ABNUM: loop {
            command = if let Some(c) = command_stack.last_mut().unwrap().pop() {c} else {')'};	//get next character, finish number if not possible
            match command {
                '0'..='9' => {
                    dig.push(command);	//add numerals to digit
                },
                '-'|'_' => {
                    if neg {
                        output.push(output!(Err, "! Unable to parse any-base number: more than one negative sign"));
                        if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                            command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                        }
                        else {
                            command_stack.last_mut().unwrap().clear();
                        }
                        break;
                    }
                    neg = true;
                },
                '.' => {
                    if frac {
                        output.push(output!(Err, "! Unable to parse any-base number: more than one fractional separator"));
                        if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                            command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                        }
                        else {
                            command_stack.last_mut().unwrap().clear();
                        }
                        break;
                    }
                    frac = true;
                    command_stack.last_mut().unwrap().push(' ');	//end digit in next iteration
                },
                '@' => {
                    exp = true;
                    command_stack.last_mut().unwrap().push(' ');	//end digit in next iteration, exponent handled by finalizer
                },
                ' '|')' => {	//if digit or whole number is finished
                    let digint = if dig.clone().is_empty() {Integer::ZERO} else {Integer::parse(dig.clone()).unwrap().complete()};	//parse digit, default to 0
                    if digint >= ibase {
                        output.push(output!(Err, format!("! Unable to parse any-base number: digit '{}' is too high for base {}", digint, ibase)));
                        if command==')' {break;}
                        else {
                            if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                                command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                            }
                            else {
                                command_stack.last_mut().unwrap().clear();
                            }
                            break;
                        }
                    }
                    num *= ibase.clone();	//add digit to number: multiply old contents by radix...
                    num += digint;	//... and add new digit
                    dig.clear();
                    if frac {
                        scale *= ibase.clone();	//if fractional part has started, make scale keep up
                    }
                    let escale =	//power applied to input base for exponential notation
                    if exp {	//if exponential part has begun
                        let mut epart = String::new();
                        let mut eneg = false;
                        while !command_stack.last().unwrap().is_empty() {
                            command = command_stack.last_mut().unwrap().pop().unwrap();
                            match command {
                                '0'..='9' => {
                                    epart.push(command);
                                },
                                '-'|'_' => {
                                    if eneg {
                                        output.push(output!(Err, "! Unable to parse any-base number: more than one negative sign in exponent"));
                                        if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                                            command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                                        }
                                        else {
                                            command_stack.last_mut().unwrap().clear();
                                        }
                                        break 'CANCEL_ABNUM;
                                    }
                                    epart.insert(0, '-');
                                    eneg = true;
                                },
                                ')' => {
                                    break;
                                },
                                _ => {
                                    output.push(Err(format!("! Unable to parse any-base number: invalid character '{}' in exponent", command)));
                                    if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                                        command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                                    }
                                    else {
                                        command_stack.last_mut().unwrap().clear();
                                    }
                                    break 'CANCEL_ABNUM;
                                },
                            }
                        }
                        Integer::parse(epart).unwrap().complete()
                    }
                    else {
                        Integer::from(0 as i16)
                    };
                    if command==')' {	//if number finished, push to stack
                        if scale> (1 as i16) {
                            scale /= ibase.clone();	//correct off-by-one error
                        }
                        handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, num * if neg {-1 as i16} else {1 as i16}) / scale
                            * Float::with_val(handler.working_precision, ibase).pow(escale)));
                        break;
                    }
                },
                _ => {
                    output.push(output!(Err, format!("! Invalid character in any-base number: '{}'", command)));
                    if let Some(idx) = command_stack.last().unwrap().rfind(')') {
                        command_stack.last_mut().unwrap().truncate(idx);	//remove rest of erroneous number
                    }
                    else {
                        command_stack.last_mut().unwrap().clear();
                    }
                    break;
                },
            }
        }
    }
    output
}

//string input
pub fn string_input<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    let mut res = String::new();	//result string
    let mut nest: usize = 1;	//nesting level
    command = command_stack.last_mut().unwrap().pop().unwrap_or('\0');	//overwrite opening bracket, null if nothing left
    loop {
        res.push(command);
        if command == '[' { nest+=1; }
        if command == ']' { nest-=1; }
        if nest==0 {	//string finished
            res.pop();	//remove closing bracket
            handler.main_stack.push(StackObject::String(res));
            break;
        }
        if command_stack.last().unwrap().is_empty() {	//only reached on improper string
            output.push(output!(Err, format!("! Unable to parse string \"[{}\": missing closing bracket", res)));
            break;
        }
        else {command = command_stack.last_mut().unwrap().pop().unwrap();}
    }
    output
}
