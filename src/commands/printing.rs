use crate::handler::{Handler, Output, StackObject, Command, output};

//print top with newline
pub fn print_top<'a>(handler: &'a Handler) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if !handler.main_stack.is_empty() {
        match handler.main_stack.last().unwrap() {
            StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()))),
            StackObject::Float(float) => output.push(output!(Ok, format!("[{}]", super::float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())))),
        }
    }
    output
}

//print full stack top to bottom
pub fn print_full_stack<'a>(handler: &'a Handler) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if !handler.main_stack.is_empty() {
        for i in (0..handler.main_stack.len()).rev() {
            match handler.main_stack[i] {
                StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()))),
                StackObject::Float(float) => output.push(output!(Ok, format!("[{}]", super::float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())))),
            }
        }
    }
    output
}

//pop and print without newline
pub fn print_pop_wo_newline(handler: &Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        match a {
            StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()), Command::NoNewLine)),
            StackObject::Float(float) => output.push(output!(Ok, format!("{}", super::float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())), Command::NoNewLine)),
        }
    }
    output
}

//pop and print with newline
pub fn print_pop(handler: &Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        match a {
            StackObject::String(string) => output.push(output!(Ok, format!("{}", string.clone()))),
            StackObject::Float(float) => output.push(output!(Ok, format!("{}", super::float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())))),
        }
    }
    output
}

//print register
pub fn print_register<'a>(handler: &'a Handler, command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
        output.push(output!(Err, "! No register number provided"));
    }
    else {
        let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
        if handler.registers.len()>ri {
            if !handler.registers[ri].is_empty(){
                for i in (0..handler.registers[ri].len()).rev() {
                    output.push(output!(Ok, match handler.registers[ri][i][0] {
                         StackObject::String(string) => format!("[{}]", string.clone()),
                         StackObject::Float(float) => format!("{}", super::float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone()))
                    }));
                    if !handler.registers[ri][i].is_empty() {
                        let maxwidth = handler.registers[ri][i].len().to_string().len();	//length of longest index number
                        for ai in 0..handler.registers[ri][i].len() {
                            output.push(output!(Ok, match handler.registers[ri][i][ai] {
                                StackObject::String(string) => format!("\t{:>maxwidth$}: [{}]", ai, string),
                                StackObject::Float(float) => format!("\t{:>maxwidth$}: {}", ai, super::float_to_string(float, handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())),
                            }));
                        }
                    }
                }
            }
        }
        else {
            output.push(output!(Err, format!("! Register {} is not available", ri)));
        }
    }
    output
}