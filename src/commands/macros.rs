use crate::handler::{Handler, Output, StackObject, Command, output};
use crate::constants;

use rug::{Integer, integer::Order, Float, float::Round};

//convert least significant 32 bits to one-char string or first char of string to number
pub fn convert_type(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let mut a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.is_string() {
                if a.get_string().is_empty() {
                    output.push(output!(Err, "! Cannot convert empty string to number"));
                }
                else {
                    handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, a.get_string().remove(0) as u32)));
                }
            }
            else {
                if let Some(ia) = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0.to_u32() {
                    if let Some(res) = char::from_u32(ia) {
                        handler.main_stack.push(StackObject::String(res.to_string()));
                    }
                    else {
                        output.push(output!(Err, format!("! Unable to convert number {} to character: not a valid Unicode value", ia)));
                    }
                }
                else {
                    output.push(output!(Err, format!("! Unable to convert number {} to character: valid range is 0 to {}", a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0, u32::MAX)));
                }
            }
        }
    }
    output
}

//convert number to UTF-8 string or back
pub fn convert_utf8(handler: &mut Handler, command: char) -> Vec<Output> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.is_string() {
                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, Integer::from_digits(a.get_string().as_bytes(), Order::Msf))));
            }
            else {
                if let Ok(res) = String::from_utf8(a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0.to_digits::<u8>(Order::Msf)) {
                    handler.main_stack.push(StackObject::String(res));
                }
                else {
                    output.push(output!(Err, format!("! Unable to convert number {} to string: not valid UTF-8", a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0)));
                }
            }
        }
    }
    output
}

//execute string as macro
pub fn execute_macro<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if a.is_string() {
            if command_stack.last().unwrap().is_empty() {
                command_stack.pop();	//optimize tail call
            }
            command_stack.push(super::rev_str(a.get_string().clone()));
        }
        else {
            handler.main_stack.push(a);
        }
    }
    output
}

//invert next conditional
pub fn invert(inv: &mut bool) -> Vec<Output> {
    *inv = !*inv;
    Vec::new()
}

//conditionally execute macro
pub fn if_macro<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char, inv: &mut bool) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();	//deliberately reverse order
        let b = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            let mut mac = String::new();
            if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
                output.push(output!(Err, "! No register name provided"));
            }
            else {
                let ri = handler.direct_register_selector.take().unwrap_or(command_stack.last_mut().unwrap().pop().unwrap() as usize);
                if handler.registers.len()>ri {
                    if handler.registers[ri].is_empty() {
                        output.push(output!(Err, format!("! Register {} is empty", ri)));
                    }
                    else {
                        mac = handler.registers[ri].last().unwrap().clone().first().unwrap().get_string().clone();	//get macro if possible
                    }
                }
                else {
                    output.push(output!(Err, format!("! Register {} is not available", ri)));
                }
            }
            if !mac.is_empty() {
                if *inv != match command {	//like xor
                    '<' => { a.get_float() < b.get_float() },
                    '=' => { a.get_float() == b.get_float() },
                    '>' => { a.get_float() > b.get_float() },
                    _ => {false},
                }
                {
                    if command_stack.last().unwrap().is_empty() {
                        command_stack.pop();	//optimize tail call
                    }
                    command_stack.push(super::rev_str(mac));
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
    *inv = false;	//always reset inversion
    output
}

//auto-macro
pub fn auto_macro<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let b = handler.main_stack.pop().unwrap();
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, Some(&b), None, &mut output) {
            let int = b.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if let Some(reps) = int.to_usize() {
                if command_stack.last().unwrap().is_empty() {
                    command_stack.pop();	//optimize tail call
                }
                command_stack.resize(command_stack.len()+reps, super::rev_str(a.get_string().clone()));
            }
            else {
                output.push(output!(Err, format!("! Invalid macro repeat count: {}", int)));
            }
        }
    }
    output
}

//quit dcim
pub fn quit(handler: &mut Handler) -> Vec<Output> {
    vec![match handler.direct_register_selector {
        Some(drs) => output!(Ok, "Exiting...", vec![Command::Exit(drs as i32)]),
        _ => output!(Ok, "Exiting...", vec![Command::Exit(0)]),
    }]
}

//quit a macro calls
pub fn quit_macro<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            let int = a.get_float().to_integer_round(Round::Zero).unwrap_or(constants::env::INT_ROUND_DEFAULT).0;
            if let Some(mut num) = int.to_usize() {
                if num>command_stack.len() {num=command_stack.len();}
                command_stack.truncate(command_stack.len()-num);
                if command_stack.is_empty() {
                    command_stack.push(String::new());	//guarantee at least one object
                }
            }
            else {
                output.push(output!(Err, format!("! Cannot possibly quit {} levels", int)));
            }
        }
    }
    output
}

//prompt and execute
pub fn prompt<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    //TODO Figure Out How to Handle Prompts
    /*
    let mut prompt_in = String::new();
    stdin().read_line(&mut prompt_in).expect("Unable to read input");
    prompt_in = prompt_in.trim_end_matches(char::is_whitespace).to_string();	//trim trailing LF
    if command_stack.last().unwrap().is_empty() {
        command_stack.pop();	//optimize tail call
    }
    command_stack.push(rev_str(prompt_in));
    */
    vec![output!(Err, "! Prompts are not yet supported")]
}

//execute file as script
pub fn execute_as_script<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    //TODO Figure Out How to Handle Execute As Script
    /*
    if check_n(command, handler.main_stack.len()) {
        let a = handler.main_stack.pop().unwrap();
        if check_t(command, a.t, false, false) {
            match std::fs::read_to_string(a.s.clone()) {
                Ok(script) => {
                    let mut script_nc = String::new();	//script with comments removed
                    for line in script.split('\n') {
                        script_nc.push_str(line.split_once('#').unwrap_or((line,"")).0);	//remove comment on every line
                        script_nc.push('\n');
                    }
                    command_stack.push(rev_str(script_nc));
                },
                Err(error) => {
                    output.push(output!(Err, format!("! Unable to read file \"{}\": {}", a.s, error)));
                },
            }
        }
    }
    */
    vec![output!(Err, "! Execute As Script is not yet supported")]
}

//get environment variable
pub fn get_env<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    //TODO Figure Out How to Handle Get Environment Variable
    /*
    let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            match std::env::var(&a.get_string()) {
                Ok(val) => {
                    handler.main_stack.push(Obj::s(val));
                },
                Err(err) => {
                    output.push(output!(Err, format!("! Unable to get value of \"{}\": {}", a.s, err)));
                },
            }
        }
    }
    output
    */
    vec![output!(Err, "! Get Environment Variable is not yet supported")]
}

//execute os command(s)
pub fn execute_os<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char) -> Vec<Output<'a>> {
    //TODO Figure Out How to Handle Execute OS
    /*
    if check_n(command, handler.main_stack.len()) {
        let a = handler.main_stack.pop().unwrap();
        if check_t(command, a.t, false, false) {
            for oscmd in a.s.split(';') {
                if let Some((var, val)) = oscmd.split_once('=') {	//set variable
                    std::env::set_var(var, val);
                }
                else {	//normal command
                    let mut args: Vec<&str> = oscmd.trim().split(' ').collect();
                    match std::process::Command::new(args.remove(0)).args(args).spawn() {
                        Ok(mut child) => {
                            if let Ok(stat) = child.wait() {
                                if let Some(code) = stat.code() {
                                    if code!=0 {output.push(output!(Err, format!("! OS command \"{}\" exited with code {}", oscmd, code)));}
                                }
                            }
                        },
                        Err(error) => {
                            output.push(output!(Err, format!("! Unable to execute OS command \"{}\": {}", oscmd, error)));
                        },
                    }
                }
            }
        }
    }
    
    */
    vec![output!(Err, "! Execute OS is not yet supported")]
}

//stop on beginning of #comment
pub fn stop_on_comment<'a>(command_stack: &mut Vec<String>) -> Vec<Output<'a>> {
    command_stack.last_mut().unwrap().clear();
    Vec::new()
}