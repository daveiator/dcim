use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow, rand::RandState};

use crate::handler::{Handler, Output, StackObject, output};

pub fn execute(handler: &mut Handler, expression: String) -> Vec<Output> {
    let mut output: Vec<Output> = Vec::new();
    let mut command_stack: Vec<String> = if expression.is_empty() {
        Vec::new()
    } else {
        vec![rev_str(expression)]
    };
    while !command_stack.is_empty() {
        let command = command_stack.pop().unwrap().pop().unwrap();
    
        match command {
			/*------------------
				OBJECT INPUT
			------------------*/
			//standard number input, force with single quote to use letters
			'0'..='9'|'.'|'_'|'\''|'@' => {
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
			},

			//any-base number input
			'(' => {
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
			},

			//string input
			'[' => {
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
			},
			/*--------------
				PRINTING
			--------------*/
			//print top with newline
			'p' => {
				if !handler.main_stack.is_empty() {
                    match handler.main_stack.last().unwrap() {
                        StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()))),
                        StackObject::Float(float) => output.push(output!(Ok, format!("[{}]", float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())))),
                    }
				}
			},

			//print full stack top to bottom
			'f' => {
				if !handler.main_stack.is_empty() {
					for i in (0..handler.main_stack.len()).rev() {
						match handler.main_stack[i] {
                            StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()))),
                            StackObject::Float(float) => output.push(output!(Ok, format!("[{}]", float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())))),
                        }
					}
				}
			},

			//pop and print without newline
			'n' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
                    match a {
                        StackObject::String(string) => output.push(output!(Ok, format!("[{}]", string.clone()), Command::NoNewLine)),
                        StackObject::Float(float) => output.push(output!(Ok, float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone()), Command::NoNewLine)),
                    }
				}
			},

			//pop and print with newline
			'P' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					match a {
                        StackObject::String(string) => output.push(output!(Ok, format!("{}", string.clone()))),
                        StackObject::Float(float) => output.push(output!(Ok, float_to_string(float.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone()))),
                    }
				}
			},

			//print register
			'F' => {
				if command_stack.last().unwrap().is_empty() && handler.direct_register_selector == None {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if let Some(_) = handler.direct_register_selector {
						handler.direct_register_selector.take().unwrap()
					} else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						if !handler.registers[ri].is_empty(){
							for i in (0..handler.registers[ri].len()).rev() {
								if handler.registers[ri][i].o.t {
									output.push(output!(Ok, format!("[{}]", handler.registers[ri][i].o.s.clone())));
								}
								else {
									output.push(output!(Ok, format!("{}", flt_to_str(handler.registers[ri][i].o.n.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone()))));
								}
								if !handler.registers[ri][i].a.is_empty() {
									let maxwidth = handler.registers[ri][i].a.len().to_string().len();	//length of longest index number
									for ai in 0..handler.registers[ri][i].a.len() {
										if handler.registers[ri][i].a[ai].t {
											output.push(output!(Ok, format!("\t{:>maxwidth$}: [{}]", ai, handler.registers[ri][i].a[ai].s)));
										}
										else {
											output.push(output!(Ok, format!("\t{:>maxwidth$}: {}", ai, flt_to_str(handler.registers[ri][i].a[ai].n.clone(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone()))));
										}
									}
								}
							}
						}
					}
					else {
						output.push(output!(Err, format!("! Register {} is not available", ri)));
					}
				}
			},
			/*----------------
				ARITHMETIC
			----------------*/
			//add or concatenate strings
			'+' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						//concat strings
						if a.t {
							handler.main_stack.push(Obj::s(a.s + &b.s));
						}
						//add numbers
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n + b.n)));
						}
					}
				}
			},

			//subtract or remove chars from string
			'-' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						//remove b chars from string a
						if a.t {
							let mut newstr = a.s.chars().collect::<Vec<char>>();
							let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;	//extract b, keep for checking if negative
							if let Some(mut num) = &int.abs_ref().complete().to_usize() {
								if num>newstr.len() { num = newstr.len(); }	//account for too large b
								if int<0 { newstr.reverse(); }	//if negative, reverse to remove from front
								newstr.truncate(newstr.len()-num);
								if int<0 { newstr.reverse(); }	//undo reversal
								handler.main_stack.push(Obj::s(newstr.iter().collect::<String>()));
							}
							else {
								output.push(output!(Err, format!("! Cannot possibly remove {} characters from a string", int)));
							}
						}
						//subtract numbers
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n - b.n)));
						}
					}
				}
			},

			//multiply or repeat/invert string
			'*' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						//repeat string a b times
						if a.t {
							let mut newstr = a.s;
							let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;	//extract b, keep for checking if negative
							if let Some(mut num) = &int.abs_ref().complete().to_usize() {
								if num*newstr.len()>usize::MAX { num = usize::MAX/newstr.len(); }	//account for too large b
								newstr = newstr.repeat(num);
								if int<0 { newstr = rev_str(newstr); }	//if b is negative, invert string
								handler.main_stack.push(Obj::s(newstr));
							}
							else {
								output.push(output!(Err, format!("! Cannot possibly repeat a string {} times", int)));
							}
						}
						//multiply numbers
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n * b.n)));
						}
					}
				}
			},
			
			//divide or shorten string to length
			'/' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						//shorten string a to length b
						if a.t {
							let mut newstr = a.s.chars().collect::<Vec<char>>();
							let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;	//extract b, keep for checking if negative
							if let Some(num) = &int.abs_ref().complete().to_usize() {
								if int<0 { newstr.reverse(); }	//if negative, reverse to remove from front
								newstr.truncate(*num);
								if int<0 { newstr.reverse(); }	//undo reversal
								handler.main_stack.push(Obj::s(newstr.iter().collect::<String>()));
							}
							else {
								output.push(output!(Err, format!("! Cannot possibly shorten a string to {} characters", int)));
							}
						}
						//divide numbers
						else {
							if b.n==0 {
								output.push(output!(Err, "! Arithmetic error: Attempted division by zero"));
							}
							else {
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n / b.n)));
							}
						}
					}
				}
			},

			//modulo, integers only
			'%' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						let ia = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						let ib = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if ib==0 {
							output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, ia % ib)));
						}
					}
				}
			},

			//euclidean division or split string
			'~' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						if a.t {
							let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
							if let Some(mut idx) = &int.to_usize() {
								let cvec = a.s.chars().collect::<Vec<char>>();
								if idx>cvec.len() { idx=cvec.len(); }	//if too large, split at max index to preserve signature
								handler.main_stack.push(Obj::s(cvec[0..idx].iter().collect::<String>()));
								handler.main_stack.push(Obj::s(cvec[idx..].iter().collect::<String>()));
							}
							else {
								output.push(output!(Err, format!("! Cannot possibly split a string at character {}", int)));
							}
						}
						else {
							let ia = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
							let ib = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
							if ib==0 {
								output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
							}
							else {
								let (quot, rem)=ia.div_rem_euc(ib);
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, quot)));
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, rem)));
							}
						}
					}
				}
			},

			//exponentiation
			'^' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						if a.n<0&&b.n.clone().abs()<1{
							output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.pow(b.n))));
						}
					}
				}
			},

			//modular exponentiation, integers only
			'|' => {
				if check_n(command, handler.main_stack.len()) {
					let c = handler.main_stack.pop().unwrap();
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, c.t) {
						let ia = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						let ib = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						let ic = c.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if ic==0 {
							output.push(output!(Err, "! Arithmetic error: Attempted reduction mod 0"));
						}
						else {
							if let Ok(res) = ia.clone().pow_mod(&ib, &ic) {
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, res)));
							}
							else {
								output.push(output!(Err, format!("! Arithmetic error: {} doesn't have an inverse mod {}", ia, ic)));
							}
						}
					}
				}
			},

			//square root
			'v' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.n<0 {
							output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.sqrt())));
						}
					}
				}
			},

			//bth root
			'V' => {
				if check_n(command, handler.main_stack.len()){
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						if a.n<0&&b.n.clone().abs()>1{
							output.push(output!(Err, "! Arithmetic error: Roots of negative numbers are not allowed"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.pow(b.n.recip()))));
						}
					}
				}
			},

			//length of string or natural logarithm
			'g' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.t {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.s.chars().count())));
						}
						else {
							if a.n<=0 {
								output.push(output!(Err, "! Arithmetic error: Logarithms of zero and negative numbers are not allowed"));
							}
							else {
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.ln())));
							}
						}
					}
				}
			},

			//base b logarithm
			'G' => {
				if check_n(command, handler.main_stack.len()){
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						if a.n<=0 {
							output.push(output!(Err, "! Arithmetic error: Logarithms of zero and negative numbers are not allowed"));
						}
						else if b.n==1||b.n<=0{
							output.push(output!(Err, "! Arithmetic error: Logarithm base must be positive and not equal to 1"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.ln()/b.n.ln())));
						}
					}
				}
			},

			//sine
			'u' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.sin())));
					}
				}
			},

			//cosine
			'y' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.cos())));
					}
				}
			},

			//tangent
			't' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.tan())));
					}
				}
			},

			//arc-sine
			'U' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.n.clone().abs()>1 {
							output.push(output!(Err, "! Arithmetic error: Arc-sine of value outside [-1,1]"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.asin())));
						}
					}
				}
			},

			//arc-cosine
			'Y' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.n.clone().abs()>1 {
							output.push(output!(Err, "! Arithmetic error: Arc-cosine of value outside [-1,1]"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.acos())));
						}
					}
				}
			},

			//arc-tangent
			'T' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.n.atan())));
					}
				}
			},

			//random integer [0;a)
			'N' => {
				if check_n(command, handler.main_stack.len()){
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int<=0 {
							output.push(output!(Err, "! Upper bound for random value must be above 0"));
						}
						else {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, int.random_below(&mut RNG[0]))));
						}
					}
				}
			},

			//constant/conversion factor lookup or convert number to string
			'"' => {
				if check_n(command, handler.main_stack.len()) {
					let mut a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.t {	//constant lookup
							match a.s.matches(' ').count() {
								0 => {	//normal lookup
									let mut scale = String::new();
									while a.s.starts_with(|c: char| c.is_ascii_digit()||c=='-') {
										scale.push(a.s.remove(0));	//extract scale prefix
									}
									if scale.is_empty() {scale.push('0');}

									let mut power = String::new();
									while a.s.ends_with(|c: char| c.is_ascii_digit()) {
										power.insert(0, a.s.pop().unwrap());	//extract power suffix
									}
									if power.is_empty() {power.push('1');}

									if let Some(res) = constants::get_prec(handler.working_precision, a.s) {
										handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, (res*Float::with_val(handler.working_precision, Integer::parse(scale).unwrap().complete()).exp10())
												.pow(Integer::parse(power).unwrap().complete()))));
									}
								},
								1 => {	//conversion shorthand, everything is like the 0 case but twice
									let (from, to) = a.s.split_once(' ').unwrap();
									let mut sfrom = String::from(from);	//convert from this
									let mut sto = String::from(to);	//to this

									let mut kfrom = String::new();
									while sfrom.starts_with(|c: char| c.is_ascii_digit()||c=='-') {
										kfrom.push(sfrom.remove(0));	//extract scale prefix
									}
									if kfrom.is_empty() {kfrom.push('0');}
									let mut pfrom = String::new();
									while sfrom.ends_with(|c: char| c.is_ascii_digit()) {
										pfrom.insert(0, sfrom.pop().unwrap());	//extract power suffix
									}
									if pfrom.is_empty() {pfrom.push('1');}

									let mut kto = String::new();
									while sto.starts_with(|c: char| c.is_ascii_digit()||c=='-') {
										kto.push(sto.remove(0));	//extract scale prefix
									}
									if kto.is_empty() {kto.push('0');}
									let mut pto = String::new();
									while sto.ends_with(|c: char| c.is_ascii_digit()) {
										pto.insert(0, sto.pop().unwrap());	//extract power suffix
									}
									if pto.is_empty() {pto.push('1');}

									if let Some(nfrom) = constants::get_prec(handler.working_precision, sfrom.to_string()) {
										if let Some(nto) = constants::get_prec(handler.working_precision, sto.to_string()) {
											handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision,
													(nfrom*Float::with_val(handler.working_precision, Integer::parse(kfrom).unwrap().complete()).exp10())
														.pow(Integer::parse(pfrom).unwrap().complete())/
													(nto*Float::with_val(handler.working_precision, Integer::parse(kto).unwrap().complete()).exp10())
														.pow(Integer::parse(pto).unwrap().complete()))));
										}
									}
								},
								_ => {
									output.push(output!(Err, format!("! Too many spaces in constant lookup/unit conversion string \"{}\"", a.s)));
								},
							}
						}
						else {	//"print" number to string
							handler.main_stack.push(Obj::s(flt_to_str(a.n, handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())));
						}
					}
				}
			},
			/*------------------------
				STACK MANIPULATION
			------------------------*/
			//clear stack
			'c' => {
				handler.main_stack.clear();
			},

			//remove top a objects from stack
			'C' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if let Some(mut num) = int.to_usize() {
							if num>handler.main_stack.len() { num = handler.main_stack.len(); }	//limit clear count
							handler.main_stack.truncate(handler.main_stack.len()-num);
						}
						else {
							output.push(output!(Err, format!("! Cannot possibly remove {} objects from the main stack", int)));
						}
					}
				}
			},

			//duplicate top of stack
			'd' => {
				if handler.main_stack.is_empty() {
					output.push(output!(Err, "! Nothing to duplicate"));
				}
				else {
					handler.main_stack.extend_from_within(handler.main_stack.len()-1..);
				}
			},

			//duplicate top a objects
			'D' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
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
			},

			//swap top 2 objects
			'r' => {
				if handler.main_stack.len()>=2 {
					handler.main_stack.swap(handler.main_stack.len()-2, handler.main_stack.len()-1);
				}
				else {
					output.push(output!(Err, "! Not enough objects to rotate"));
				}
			},

			//rotate top a objects
			'R' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let mut int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int==0 { int = Integer::from(1); }	//replace 0 with effective no-op
						if let Some(num) = int.clone().abs().to_usize() {
							if num<=handler.main_stack.len() {
								let sl = handler.main_stack.as_mut_slice();
								if int<0 {
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
			},

			//push stack depth
			'z' => {
				handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.main_stack.len())));
			},
			/*----------------------------
				ENVIRONMENT PARAMETERS
			----------------------------*/
			//set output precision
			'k' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int>=-1 {
							handler.parameter_stack.last_mut().unwrap().0 = int;
						}
						else {
							output.push(output!(Err, "! Output precision must be at least -1"));
						}
					}
				}
			},

			//set input base
			'i' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int>=2 {
							handler.parameter_stack.last_mut().unwrap().1 = int;
						}
						else {
							output.push(output!(Err, "! Input base must be at least 2"));
						}
					}
				}
			},

			//set output base
			'o' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int>=2 {
							handler.parameter_stack.last_mut().unwrap().2 = int;
						}
						else {
							output.push(output!(Err, "! Output base must be at least 2"));
						}
					}
				}
			},

			//set working precision
			'w' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if int>=1 && int<=u32::MAX {
							handler.working_precision = int.to_u32().unwrap();
						}
						else {
							output.push(output!(Err, format!("! Working precision must be between {} and {} (inclusive)", 1, u32::MAX)));
						}
					}
				}
			},

			//push output precision
			'K' => {
				handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().0.clone())));
			},

			//push input base
			'I' => {
				handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().1.clone())));
			},

			//push output base
			'O' => {
				handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.parameter_stack.last().unwrap().2.clone())));
			},

			//push working precision
			'W' => {
				handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.working_precision)));
			},

			//create new k,i,o context
			'{' => {
				handler.parameter_stack.push((kdef(), idef(), odef()));
			},

			//revert to previous context
			'}' => {
				handler.parameter_stack.pop();
				if handler.parameter_stack.is_empty() {
					handler.parameter_stack.push((kdef(), idef(), odef()));	//ensure 1 entry always remains
				}
			},
			/*--------------------------
				REGISTERS AND MACROS
			--------------------------*/
			//save to top of register
			's' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();					
					if command_stack.last().unwrap().is_empty()&&!DRS_EN {
						output.push(output!(Err, "! No register number provided"));
					}
					else {
						let ri = if DRS_EN {
							DRS_EN = false;
							DRS
						}
						else {
							command_stack.last_mut().unwrap().pop().unwrap() as usize
						};
						if handler.registers.len()>ri {
							if handler.registers[ri].is_empty() {
								handler.registers[ri].push(RegObj {
									o: a,
									a: Vec::new()
								});
							}
							else {
								handler.registers[ri].last_mut().unwrap().o = a;
							}
						}
						else {
							output.push(output!(Err, format!("! Register {} is not available", ri)));
						}
					}
				}
				else {
					if !DRS_EN {
						command_stack.last_mut().unwrap().pop();	//remove register name
					}
					DRS_EN = false;	//invalidate DRS
				}
			},

			//push to top of register
			'S' => {
				if check_n(command, handler.main_stack.len()) {
					let a=RegObj {
						o: handler.main_stack.pop().unwrap(),
						a: Vec::new()
					};
					if command_stack.last().unwrap().is_empty()&&!DRS_EN {
						output.push(output!(Err, "! No register number provided"));
					}
					else {
						let ri = if DRS_EN {
							DRS_EN = false;
							DRS
						}
						else {
							command_stack.last_mut().unwrap().pop().unwrap() as usize
						};
						if handler.registers.len()>ri {
							handler.registers[ri].push(a);
						}
						else {
							output.push(output!(Err, format!("! Register {} is not available", ri)));
						}
					}
				}
				else {
					if !DRS_EN {
						command_stack.last_mut().unwrap().pop();	//remove register name
					}
					DRS_EN = false;	//invalidate DRS
				}
			},

			//load from top of register
			'l' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						if registers[ri].is_empty() {
							output.push(output!(Err, format!("! Register {} is empty", ri)));
						}
						else {
							handler.main_stack.push(handler.registers[ri].last().unwrap().0.clone());
						}
					}
					else {
						output.push(Err, format!("! Register {} is not available", ri));
					}
				}
			},

			//pop from top of register
			'L' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						if handler.registers[ri].is_empty() {
							output.push(output!(Err, "! Register {} is empty", ri));
						}
						else {
							handler.main_stack.push(handler.registers[ri].pop().unwrap().o);
						}
					}
					else {
						output.push(output!(Err, "! Register {} is not available", ri));
					}
				}
			},

			//save to top-of-register's array
			':' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						if command_stack.last().unwrap().is_empty()&&!DRS_EN {
							output.push(output!(Err, "! No register number provided"));
						}
						else {
							let ri = if DRS_EN {
								DRS_EN = false;
								DRS
							}
							else {
								command_stack.last_mut().unwrap().pop().unwrap() as usize
							};
							if handler.registers.len()>ri {
								if handler.registers[ri].is_empty() {
									handler.registers[ri].push(RegObj {
										o: Obj::n(Float::with_val(handler.working_precision, 0)),	//create default register object if empty
										a: Vec::new()
									});
								}
								let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
								if let Some(rai) = int.to_usize() {
									if rai>=handler.registers[ri].last().unwrap().a.len() {
										handler.registers[ri].last_mut().unwrap().a.resize(rai+1, Obj::n(Float::with_val(handler.working_precision, 0)));	//extend if required, initialize with default objects
									}
									handler.registers[ri].last_mut().unwrap().a[rai] = a;
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
						if !DRS_EN {
							command_stack.last_mut().unwrap().pop();	//remove register name
						}
						DRS_EN = false;	//invalidate DRS
					}
				}
				else {
					if !DRS_EN {
						command_stack.last_mut().unwrap().pop();	//remove register name
					}
					DRS_EN = false;	//invalidate DRS
				}
			},

			//load from top-of-register's array
			';' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if command_stack.last().unwrap().is_empty()&&!DRS_EN {
							output.push(output!(Err, "! No register number provided"));
						}
						else {
							let ri = if DRS_EN {
								DRS_EN = false;
								DRS
							}
							else {
								command_stack.last_mut().unwrap().pop().unwrap() as usize
							};
							if handler.registers.len()>ri {
								if handler.registers[ri].is_empty() {
									handler.registers[ri].push(RegObj {
										o: Obj::n(Float::with_val(handler.working_precision, 0)),	//create default register object if empty
										a: Vec::new()
									});
								}
								let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
								if let Some(rai) = int.to_usize() {
									if rai>=handler.registers[ri].last().unwrap().a.len() {
										handler.registers[ri].last_mut().unwrap().a.resize(rai+1, Obj::n(Float::with_val(handler.working_precision, 0)));	//extend if required, initialize with default objects
									}
									handler.main_stack.push(handler.registers[ri].last().unwrap().a[rai].clone());
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
						if !DRS_EN {
							command_stack.last_mut().unwrap().pop();	//remove register name
						}
						DRS_EN = false;	//invalidate DRS
					}
				}
				else {
					if !DRS_EN {
						command_stack.last_mut().unwrap().pop();	//remove register name
					}
					DRS_EN = false;	//invalidate DRS
				}
			},

			//load top-of-reg into buffer
			'j' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						if handler.registers[ri].is_empty() {
							output.push(output!(Err, format!("! Register {} is empty", ri)));
						}
						else {
							RO_BUF[0] = handler.registers[ri].last().unwrap().clone();
						}
					}
					else {
						output.push(output!(Err, format!("! Register {} is not available", ri)));
					}
				}
			},

			//pop top-of-reg into buffer
			'J' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						if handler.registers[ri].is_empty() {
							output.push(output!(Err, format!("! Register {} is empty", ri)));
						}
						else {
							RO_BUF[0] = handler.registers[ri].pop().unwrap();
						}
					}
					else {
						output.push(output!(Err, format!("! Register {} is not available", ri)));
					}
				}
			},

			//save buffer to top-of-reg
			'h' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						handler.registers[ri].pop();
						handler.registers[ri].push(RO_BUF[0].clone());
					}
					else {
						output.push(output!(Err, "! Register {} is not available", ri));
					}
				}
			},

			//push buffer to register
			'H' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						handler.registers[ri].push(RO_BUF[0].clone());
					}
					else {
						output.push(output!(Err, "! Register {} is not available", ri));
					}
				}
			},

			//push register depth
			'Z' => {
				if command_stack.last().unwrap().is_empty()&&!DRS_EN {
					output.push(output!(Err, "! No register number provided"));
				}
				else {
					let ri = if DRS_EN {
						DRS_EN = false;
						DRS
					}
					else {
						command_stack.last_mut().unwrap().pop().unwrap() as usize
					};
					if handler.registers.len()>ri {
						handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, handler.registers[ri].len())));
					}
					else {
						output.push(output!(Err, format!("! Register {} is not available", ri)));
					}
				}
			},

			//specify manual register index
			',' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if let Some(ri) = int.to_usize() {
							if handler.registers.len()>ri {
								DRS = ri;
								DRS_EN = true;
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
			},
			/*------------
				MACROS
			------------*/

			//convert least significant 32 bits to one-char string or first char of string to number
			'a' => {
				if check_n(command, handler.main_stack.len()) {
					let mut a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.t {
							if a.s.is_empty() {
								output.push(output!(Err, "! Cannot convert empty string to number"));
							}
							else {
								handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, a.s.remove(0) as u32)));
							}
						}
						else {
							if let Some(ia) = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0.to_u32() {
								if let Some(res) = char::from_u32(ia) {
									handler.main_stack.push(Obj::s(res.to_string()));
								}
								else {
									output.push(output!(Err, format!("! Unable to convert number {} to character: not a valid Unicode value", ia)));
								}
							}
							else {
								output.push(output!(Err, format!("! Unable to convert number {} to character: valid range is 0 to {}", a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0, u32::MAX)));
							}
						}
					}
				}
			},

			//convert number to UTF-8 string or back
			'A' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						if a.t {
							handler.main_stack.push(Obj::n(Float::with_val(handler.working_precision, Integer::from_digits(a.s.as_bytes(), Order::Msf))));
						}
						else {
							if let Ok(res) = String::from_utf8(a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0.to_digits::<u8>(Order::Msf)) {
								handler.main_stack.push(Obj::s(res));
							}
							else {
								output.push(output!(Err, format!("! Unable to convert number {} to string: not valid UTF-8", a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0)));
							}
						}
					}
				}
			},

			//execute string as macro
			'x' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if a.t {
						if command_stack.last().unwrap().is_empty() {
							command_stack.pop();	//optimize tail call
						}
						command_stack.push(rev_str(a.s));
					}
					else {
						handler.main_stack.push(a);
					}
				}
			},

			//invert next conditional
			'!' => {
				inv = !inv;
			},

			//conditionally execute macro
			'<'|'='|'>' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();	//deliberately reverse order
					let b = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						let mut mac = String::new();
						if command_stack.last().unwrap().is_empty()&&!DRS_EN {
							output.push(output!(Err, "! No register name provided"));
						}
						else {
							let ri = if DRS_EN {
								DRS_EN = false;
								DRS
							}
							else {
								command_stack.last_mut().unwrap().pop().unwrap() as usize
							};
							if handler.registers.len()>ri {
								if handler.registers[ri].is_empty() {
									output.push(output!(Err, format!("! Register {} is empty", ri)));
								}
								else {
									mac = handler.registers[ri].last().unwrap().clone().o.s;	//get macro if possible
								}
							}
							else {
								output.push(output!(Err, format!("! Register {} is not available", ri)));
							}
						}
						if !mac.is_empty() {
							if inv != match command {	//like xor
								'<' => { a.n < b.n },
								'=' => { a.n == b.n },
								'>' => { a.n > b.n },
								_ => {false},
							}
							{
								if command_stack.last().unwrap().is_empty() {
									command_stack.pop();	//optimize tail call
								}
								command_stack.push(rev_str(mac));
							}
						}
					}
					else {
						if !DRS_EN {
							command_stack.last_mut().unwrap().pop();	//remove register name
						}
						DRS_EN = false;	//invalidate DRS
					}
				}
				else {
					if !DRS_EN {
						command_stack.last_mut().unwrap().pop();	//remove register name
					}
					DRS_EN = false;	//invalidate DRS
				}
				inv = false;	//always reset inversion
			},

			//auto-macro
			'X' => {
				if check_n(command, handler.main_stack.len()) {
					let b = handler.main_stack.pop().unwrap();
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, b.t, false) {
						let int = b.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if let Some(reps) = int.to_usize() {
							if command_stack.last().unwrap().is_empty() {
								command_stack.pop();	//optimize tail call
							}
							command_stack.resize(command_stack.len()+reps, rev_str(a.s));
						}
						else {
							output.push(output!(Err, formtat!("! Invalid macro repeat count: {}", int)));
						}
					}
				}
			},

			//quit dcim
			'q' => {
				std::process::exit(if DRS_EN {DRS as i32} else {0});
			},

			//quit a macro calls
			'Q' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						let int = a.n.to_integer_round(Round::Zero).unwrap_or(INT_ORD_DEF).0;
						if let Some(mut num) = int.to_usize() {
							if num>command_stack.len() {num=command_stack.len();}
							command_stack.truncate(command_stack.len()-num);
							if command_stack.is_empty() {
								command_stack.push(String::new());	//guarantee at least one object
							}
						}
						else {
							output.push(output!(Err, formtat!("! Cannot possibly quit {} levels", int)));
						}
					}
				}
			},

			//prompt and execute
			'?' => {
				let mut prompt_in = String::new();
				stdin().read_line(&mut prompt_in).expect("Unable to read input");
				prompt_in = prompt_in.trim_end_matches(char::is_whitespace).to_string();	//trim trailing LF
				if command_stack.last().unwrap().is_empty() {
					command_stack.pop();	//optimize tail call
				}
				command_stack.push(rev_str(prompt_in));
			},

			//execute file as script
			'&' => {
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
			},

			//get environment variable
			'$' => {
				if check_n(command, handler.main_stack.len()) {
					let a = handler.main_stack.pop().unwrap();
					if check_t(command, a.t, false, false) {
						match std::env::var(&a.s) {
							Ok(val) => {
								handler.main_stack.push(Obj::s(val));
							},
							Err(err) => {
								output.push(output!(Err, format!("! Unable to get value of \"{}\": {}", a.s, err)));
							},
						}
					}
				}
			},

			//execute os command(s)
			'\\' => {
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
			},

			//stop on beginning of #comment
			'#' => {
				command_stack.last_mut().unwrap().clear();
			},

			//notify on invalid command, keep going
			_ => {
				if !command.is_whitespace()&&command!='\0' { output.push(output!(Err, format!("! Invalid command: {} (U+{:04X})", command, command as u32))); }
			},
		}
		while let Some(ptr) = command_stack.last() {
			if ptr.is_empty() {
				command_stack.pop();
			}
			else{break;}
		}
        
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


//checks if n arguments are sufficient for a command (defines adicity)
//not used by niladics
fn check_n(op: char, n: usize) -> bool {
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