use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::RandState};

use crate::handler::{Handler, Output, StackObject, output};

mod object_input;
mod printing;
mod arithmetic;
mod constants;
mod stack_manipulation;
mod environment;
mod register;
mod macros;

fn run_command<'a>(handler: &'a mut Handler, command_stack: &mut Vec<String>, command: char, inv: &mut bool) -> Vec<Output<'a>> {
	match command {
		/*------------------
			OBJECT INPUT
		------------------*/
		//standard number input, force with single quote to use letters
		'0'..='9'|'.'|'_'|'\''|'@' => object_input::standard_number_input(handler, command_stack, command),
		//any-base number input
		'(' => object_input::any_base_number_input(handler, command_stack, command),
		//string input
		'[' => object_input::string_input(handler, command_stack, command),
	
		/*--------------
			PRINTING
		--------------*/
		//print top with newline
		'p' => printing::print_top(handler),
		//print full stack top to bottom
		'f' => printing::print_full_stack(handler),
		//pop and print without newline
		'n' => printing::print_pop_wo_newline(handler, command),
		//pop and print with newline
		'P' => printing::print_pop(handler, command),
		//print register
		'F' => printing::print_register(handler, command_stack),
	
		/*----------------
			ARITHMETIC
		----------------*/
		//add or concatenate strings
		'+' => arithmetic::add(handler, command),
		//subtract or remove chars from string
		'-' => arithmetic::subtract(handler, command),
		//multiply or repeat/invert string
		'*' => arithmetic::multiply(handler, command),
		//divide or shorten string to length
		'/' => arithmetic::divide(handler, command),
		//modulo, integers only
		'%' => arithmetic::modulo(handler, command),
		//euclidean division or split string
		'~' => arithmetic::euclidean_division(handler, command),
		//exponentiation
		'^' => arithmetic::exponent(handler, command),
		//modular exponentiation, integers only
		'|' => arithmetic::modular_exponent(handler, command),
		//square root
		'v' => arithmetic::square_root(handler, command),
		//nth root
		'V' => arithmetic::nth_root(handler, command),
		//length of string or natural logarithm
		'g' => arithmetic::ln(handler, command),
		//base b logarithm
		'G' => arithmetic::log(handler, command),
		//sine
		'u' => arithmetic::sin(handler, command),
		//cosine
		'y' => arithmetic::cos(handler, command),
		//tangent
		't' => arithmetic::tan(handler, command),
		//arc-sine
		'U' => arithmetic::asin(handler, command),
		//arc-cosine
		'Y' => arithmetic::acos(handler, command),
		//arc-tangent
		'T' => arithmetic::atan(handler, command),
		//random integer [0;a)
		'N' => arithmetic::random_int(handler, command),

		//constant/conversion factor lookup or convert number to string
		'"' => constants::lookup(handler, command),

		/*------------------------
			STACK MANIPULATION
		------------------------*/
		//clear stack
		'c' => stack_manipulation::clear_stack(handler),
		//remove top a objects from stack
		'C' => stack_manipulation::remove_objects(handler, command),
		//duplicate top of stack
		'd' => stack_manipulation::duplicate_object(handler),
		//duplicate top a objects
		'D' => stack_manipulation::duplicate_objects(handler, command),
		//swap top 2 objects
		'r' => stack_manipulation::swap_objects(handler),
		//rotate top a objects
		'R' => stack_manipulation::rotate_objects(handler, command),
		//push stack depth
		'z' => stack_manipulation::push_stack_depth(handler),

		/*----------------------------
			ENVIRONMENT PARAMETERS
		----------------------------*/
		//set output precision
		'k' => environment::set_output_precision(handler, command),
		//set input base
		'i' => environment::set_input_base(handler, command),
		//set output base
		'o' => environment::set_output_base(handler, command),
		//set working precision
		'w' => environment::set_working_precision(handler, command),
		//push output precision
		'K' => environment::push_output_precision(handler),
		//push input base
		'I' => environment::push_input_base(handler),
		//push output base
		'O' => environment::push_output_base(handler),
		//push working precision
		'W' => environment::push_working_precision(handler),
		//create new k,i,o context
		'{' => environment::new_context(handler),
		//revert to previous context
		'}' => environment::revert_context(handler),

		/*--------------------------
			REGISTERS AND MACROS
		--------------------------*/
		//save to top of register
		's' => register::save_to_register(handler, command_stack, command),
		//push to top of register
		'S' => register::push_to_register(handler, command_stack, command),
		//load from top of register
		'l' => register::load_from_register(handler, command_stack, command),
		//pop from top of register
		'L' => register::pop_from_register(handler, command_stack, command),
		//save to top-of-register's array
		':' => register::save_to_top_of_register(handler, command_stack, command),
		//load from top-of-register's array
		';' => register::load_from_top_of_register(handler, command_stack, command),
		//load top-of-reg into buffer
		'j' => register::load_top_of_register_into_buffer(handler, command_stack),
		//pop top-of-reg into buffer
		'J' => register::pop_top_of_register_into_buffer(handler, command_stack),
		//save buffer to top-of-reg
		'h' => register::save_buffer_to_top_of_register(handler, command_stack),
		//push buffer to register
		'H' => register::push_buffer_to_register(handler, command_stack),
		//push register depth
		'Z' => register::push_register_depth(handler, command_stack),
		//specify manual register index
		',' => register::register_index(handler, command_stack, command),

		/*------------
			MACROS
		------------*/
		//convert least significant 32 bits to one-char string or first char of string to number
		'a' => macros::convert_type(handler, command),
		//convert number to UTF-8 string or back
		'A' => macros::convert_utf8(handler, command),
		//execute string as macro
		'x' => macros::execute_macro(handler, command_stack, command),
		//invert next conditional
		'!' => macros::invert(inv),
		//conditionally execute macro
		'<'|'='|'>' => macros::if_macro(handler, command_stack, command, inv),
		//auto-macro
		'X' => macros::auto_macro(handler, command_stack, command),
		//quit dcim
		'q' => macros::quit(handler),
		//quit a macro calls
		'Q' => macros::quit_macro(handler, command_stack, command),
		//prompt and execute //TODO implement
		'?' => macros::prompt(handler, command_stack, command),
		//execute file as script //TODO implement
		'&' => macros::execute_as_script(handler, command_stack, command),
		//get environment variable //TODO implement
		'$' => macros::get_env(handler, command_stack, command),
		//execute os command(s) //TODO implement
		'\\' => macros::execute_os(handler, command_stack, command),
		//stop on beginning of #comment
		'#' => macros::stop_on_comment(command_stack),

		//notify on invalid command, keep going
		_ => {
			let mut output = Vec::new();
			if !command.is_whitespace()&&command!='\0' {
				output.push(output!(Err, format!("! Invalid command: {} (U+{:04X})", command, command as u32)));
			}
			output
		},
	}
}

pub fn execute(handler: &mut Handler, expression: String) -> Vec<Output> {
    let mut output: Vec<Output> = Vec::new();
	let mut inv = false;	//invert next comparison
    let mut command_stack: Vec<String> = if expression.is_empty() {
        Vec::new()
    } else {
        vec![rev_str(expression)]
    };
    while !command_stack.is_empty() {
        let command = command_stack.pop().unwrap().pop().unwrap();

		output.extend(run_command(handler, &mut command_stack, command, &mut inv));
		inv = false;
		while let Some(ptr) = command_stack.last() {
			if ptr.is_empty() {
				command_stack.pop();
			}
			else{break;}
		}
        
    }
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

//checks if n arguments are sufficient for a command (defines adicity)
//not used by niladics
fn check_n(op: char, n: usize, output: &mut Vec<Output>) -> bool {
	if match op {
		//triadic
		'|' => n>=3,

		//dyadic
		'+'|'-'|'*'|'/'|'^'|'V'|'G'|'%'|'~'|'@'|':'|'='|'<'|'>'|'X' => n>=2,

		//monadic unless specified
		_ => n>=1,
	}
	{ true }
	else {
		output.push(output!(Err, format!("! Insufficient arguments for command '{}'", op)));
		false
	}
}

//checks if a command can be used on provided argument types
//a-c: types (.t) of the operands that would be used (in canonical order), use false if not required
fn check_t(op: char, a: &StackObject, b: Option<&StackObject>, c: Option<&StackObject>, output: &mut Vec<Output>) -> bool {
	let a = a.is_string();
	let b = b.map(|x| x.is_string()).unwrap_or(false);
	let c = c.map(|x| x.is_string()).unwrap_or(false);
	if 
		match op {
			//'+' can also concatenate strings
			'+' => a == b,

			//string manipulation, store into array
			'-'|'*'|'/'|'~'|':' => !b,

			//read file by name, get env variable, execute os command
			'&'|'$'|'\\' => a,

			//convert both ways, constant lookup by string name or convert number to string, execute macros, get log or string length
			'a'|'A'|'"'|'x'|'g' => true,

			//auto-macro
			'X' => a && !b,

			//all other ops can only have numbers
			_ => !a && !b && !c,
		}
	{
		true
	} else {
		output.push(output!(Err, format!("! Invalid argument type(s) for command '{}'", op)));
		false
	}
}

//custom number printing function
//if output base is over 36, prints in custom "any-base" notation
//otherwise, applies precision like dc and converts from exponential notation if not too small
fn float_to_string(mut num: Float, obase: Integer, oprec: Integer) -> String {
	if num.is_zero() {
		return String::from(if obase>36 {"(0)"} else {"0"});	//causes issues, always "0" regardless of parameters
	}
	if num.is_infinite() {
		return String::from("Infinity");
	}
	if num.is_nan() {
		return String::from("Not a number");
	}

	if obase>36 {	//any-base printing (original base conversion algorithm out of necessity, limited precision possible)
		let mut outstr = String::from(if num<0 {"(-"} else {"("});	//apply negative sign
		num = num.abs();
		let mut scale: usize = 0;	//amount to shift fractional separator in output
		while !num.is_integer()&&(oprec<0||scale<oprec) {	//turn into integer scaled by power of obase, apply output precision if enabled
			let temp = num.clone() * &obase;	//preview scale-up
			if temp.is_infinite() {	//possible with high precision due to Float's exponent limitation
				num /= &obase;	//prevent overflow in later "extra precision" part
				break;	//disregard further precision
			}
			num = temp;	//if ok, commit to scale-up
			scale +=1;
		}
		num *= &obase;	//get extra precision for last digit
		let mut int = num.to_integer().unwrap();	//convert to Integer
		int /= &obase;	//undo extra precision
		let mut dig = Integer::from(1);	//current digit value
		while dig<=int {
			dig *= &obase;	//get highest required digit value
		}
		dig /= &obase;	//correct off-by-one error
		loop {	//separate into digits
			let (quot, rem) = int.clone().div_rem_euc(dig.clone());	//separate into amount of current digit and remainder
			outstr.push_str(&quot.to_string());	//print amount of current digit
			outstr.push(' ');
			int = rem;	//switch to remainder
			if dig==1 {break;}	//stop when all digits done
			dig /= &obase;	//switch to next lower digit
		}
		if scale>0 {
			if let Some((idx, _)) = outstr.rmatch_indices(' ').nth(scale) {	//find location for fractional separator
				unsafe { outstr.as_bytes_mut()[idx] = '.' as u8; }	//and insert it
			}
			else {
				outstr.insert_str(if outstr.starts_with("(-") {2} else {1}, "0.");	//number has no integer part, add "0."
			}
		}
		outstr.pop();
		outstr.push(')');
		outstr
	}
	else {	//normal printing
		let mut outstr = num.to_string_radix(
			obase.to_i32().unwrap(),
			if oprec<0 {
				None
			}
			else {
				(oprec + Integer::from(
					num.to_integer_round(Round::Zero).unwrap().0	//integer part of num
					.to_string_radix(obase.to_i32().unwrap())	//...to string
					.trim_start_matches('-').len())).to_usize() 	//...length without negative sign, print exactly if too large
			}
		);
		if obase <= 10 {	//unify exponent symbol without searching the whole string
			let im = outstr.len()-1;	//max index
			unsafe {
				let bytes = outstr.as_bytes_mut();
				for ir in 0..=im {	//right offset
					if ir>10 {break;}	//exponents cannot have more digits, longest is @-323228496
					if bytes[im-ir]=='e' as u8 {
						bytes[im-ir] = '@' as u8;	//replace
						break;
					}
				}
			}
		}
		if outstr.starts_with('-') {
			outstr = outstr.replacen('-', "_", 1);	//replace negative sign
		}
		if outstr[if outstr.len()>11 {outstr.len()-11} else {0}..].contains('@') {	//efficiently check if in exponential notation
			let (mut mpart, epart) = outstr.rsplit_once('@').unwrap();
			mpart = mpart.trim_end_matches('0').trim_end_matches('.');	//remove trailing zeros from mantissa
			let eint = epart.parse::<i32>().unwrap();	//isolate exponential part
			if eint<0 && eint>-10 {
				outstr = "0.".to_string() + &"0".repeat(eint.abs() as usize -1) + &mpart.replacen('.', "", 1);	//convert from exponential notation if not too small
				if num<0 {
					let (ipart, fpart) = outstr.split_once('_').unwrap();
					outstr = "_".to_string() + ipart + fpart;	//move negative sign to front
				}
			}
			else {
				outstr = mpart.to_string() + "@" + &epart.replacen('-', "_", 1);	//reassemble, replace negative sign in exponent
			}
		}
		else {	//if in normal notation
			if let Some((ipart, fpart)) = outstr.split_once('.') {
				outstr = ipart.to_string() + "." + fpart.trim_end_matches('0');	//trim trailing zeros
			}
		}
		outstr.trim_end_matches('.').to_string()	//remove fractional separator
	}
}