use crate::handler::{Handler, Output};

pub trait Command {
    fn execute(&mut self, handler: &mut Handler) -> Output;
}

pub fn execute(handler: &mut Handler, expression: String) -> Vec<Output> {
    let mut output = Vec::new();
    let mut command_stack: Vec<String> = if expression.is_empty() {
        Vec::new()
    } else {
        vec![rev_str(expression)]
    };
    while !command_stack.is_empty() {
        let command = command_stack.pop().unwrap().pop().unwrap();
    }
    output.push(Err("! WIP".to_string()));
    return output;
}

//slightly more efficient string reverser, at least on my machine
fn rev_str(mut instr: String) -> String {
	let mut outstr = String::new();
	while !instr.is_empty() {
		outstr.push(instr.pop().unwrap());
	}
	outstr
}