use crate::handler::{Handler, Output, StackObject, output};
use crate::constants;

use rug::{Float, float::Round};

//save to top of register
pub fn save_to_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();					
        if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
            output.push(output!(Err, "! No register number provided"));
        }
        else {
            let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
            if handler.registers.len() > ri {
                if handler.registers[ri].is_empty() {
                    handler.registers[ri].push(vec![a]);
                }
                else {
                    let temp = handler.registers[ri].last_mut().unwrap();
                    temp = vec![a];
                }
            }
            else {
                output.push(output!(Err, format!("! Register {} is not available", ri)));
            }
        }
    }
    else {
        if handler.direct_register_selector == None {
            command_stack.last_mut().unwrap().pop();	//remove register name
        }
        handler.direct_register_selector = None;	//invalidate DRS
    }
    output
}

//push to top of register
pub fn push_to_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a= vec![handler.main_stack.pop().unwrap()];
        if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
            output.push(output!(Err, "! No register number provided"));
        }
        else {
            let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
            if handler.registers.len() > ri {
                handler.registers[ri].push(a);
            }
            else {
                output.push(output!(Err, format!("! Register {} is not available", ri)));
            }
        }
    }
    else {
        if handler.direct_register_selector == None {
            command_stack.last_mut().unwrap().pop();	//remove register name
        }
        handler.direct_register_selector = None;	//invalidate DRS
    }
    output
}

//load from top of register
pub fn load_from_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            if handler.registers[ri].is_empty() {
                output.push(output!(Err, format!("! Register {} is empty", ri)));
            }
            else {
                handler.main_stack.push(handler.registers[ri].last().unwrap().first().unwrap().clone());
            }
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//pop from top of register
pub fn pop_from_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            if handler.registers[ri].is_empty() {
                output.push(output!(Err, format!("! Register {} is empty", ri)));            }
            else {
                handler.main_stack.push(handler.registers[ri].pop().unwrap().first().unwrap().clone());
            }
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//save to top-of-register's array
pub fn save_to_top_of_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
                output.push(output!(Err, "! No register number provided"));
            }
            else {
                let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
                if handler.registers.len() > ri {
                    if handler.registers[ri].is_empty() {
                        handler.registers[ri].push(vec![StackObject::Float(Float::with_val(handler.working_precision, 0 as i8))]);	//create default register object if empty
                    }
                    let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
                    if let Some(rai) = int.to_usize() {
                        if rai>=handler.registers[ri].last().unwrap().len() {
                            handler.registers[ri].last_mut().unwrap().resize(rai+1, StackObject::Float(Float::with_val(handler.working_precision, 0 as i8)));	//extend if required, initialize with default objects
                        }
                        handler.registers[ri].last_mut().unwrap()[rai] = a;
                    }
                    else {
                        output.push(output!(Err, format!("! Cannot possibly save to array index {}", int)));
                    }
                }
                else {
                    output.push(output!(Err, format!("! Register {} is not available", ri)));
                }
            }
        }
        else {
            if handler.direct_register_selector == None {
                command_stack.last_mut().unwrap().pop();	//remove register name
            }
            handler.direct_register_selector = None;	//invalidate DRS
        }
    }
    else {
        if handler.direct_register_selector == None {
            command_stack.last_mut().unwrap().pop();	//remove register name
        }
        handler.direct_register_selector = None	//invalidate DRS
    }
    output
}

//load from top-of-register's array
pub fn load_from_top_of_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
                output.push(output!(Err, "! No register number provided"));
            }
            else {
                let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
                if handler.registers.len() > ri {
                    if handler.registers[ri].is_empty() {
                        handler.registers[ri].push(vec![StackObject::Float(Float::with_val(handler.working_precision, 0 as i8))]);	//create default register object if empty
                    }
                    let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
                    if let Some(rai) = int.to_usize() {
                        if rai>=handler.registers[ri].last().unwrap().len() {
                            handler.registers[ri].last_mut().unwrap().resize(rai+1, StackObject::Float(Float::with_val(handler.working_precision, 0 as i8)));	//extend if required, initialize with default objects
                        }
                        handler.main_stack.push(handler.registers[ri].last().unwrap()[rai].clone());
                    }
                    else {
                        output.push(output!(Err, format!("! Cannot possibly load from array index {}", int)));
                    }
                }
                else {
                    output.push(output!(Err, format!("! Register {} is not available", ri)));
                }
            }
        }
        else {
            if handler.direct_register_selector == None {
                command_stack.last_mut().unwrap().pop();	//remove register name
            }
            handler.direct_register_selector = None;	//invalidate DRS
        }
    }
    else {
        if handler.direct_register_selector == None {
            command_stack.last_mut().unwrap().pop();	//remove register name
        }
        handler.direct_register_selector = None;	//invalidate DRS
    }
    output
}

//load top-of-reg into buffer
pub fn load_top_of_register_into_buffer<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            if handler.registers[ri].is_empty() {
                output.push(output!(Err, format!("! Register {} is empty", ri)));
            }
            else {
                handler.register_buffer[0] = handler.registers[ri].last().unwrap().clone().first().unwrap().clone();
            }
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//pop top-of-reg into buffer
pub fn pop_top_of_register_into_buffer<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            if handler.registers[ri].is_empty() {
                output.push(output!(Err, format!("! Register {} is empty", ri)));
            }
            else {
                handler.register_buffer[0] = handler.registers[ri].pop().unwrap().first().unwrap().clone();
            }
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//save buffer to top-of-reg
pub fn save_buffer_to_top_of_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            handler.registers[ri].pop();
            handler.registers[ri].push(vec![handler.register_buffer[0].clone()]);
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//push buffer to register
pub fn push_buffer_to_register<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            handler.registers[ri].push(vec![handler.register_buffer[0].clone()]);
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//push register depth
pub fn push_register_depth<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len() > ri {
            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, handler.registers[ri].len())));
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}

//specify manual register index
pub fn register_index<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if let Some(ri) = int.to_usize() {
                if handler.registers.len()>ri {
                    handler.direct_register_selector = Some(ri);
                }
                else {
                    output.push(output!(Err, format!("! Register {} is not available", ri)));
                }
            }
            else {
                output.push(output!(Err, format!("! Register {} cannot possibly exist", int)));
            }
        }
    }
    output
}