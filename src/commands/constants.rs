use crate::handler::{Handler, Output, StackObject, output};
use crate::constants;

use rug::{Integer, Complete, Float, ops::Pow};

//constant/conversion factor lookup or convert number to string
pub fn lookup(handler: &mut Handler, command: char) -> Vec<Output> {
	let mut output = Vec::new();
    if super::check_n(command, handler.main_stack.len(), &mut output) {
        let mut a = handler.main_stack.pop().unwrap();
        if super::check_t(command, &a, None, None, &mut output) {
            if a.is_string() {	//constant lookup
                match a.get_string().matches(' ').count() {
                    0 => {	//normal lookup
                        let mut scale = String::new();
                        while a.get_string().starts_with(|c: char| c.is_ascii_digit()||c=='-') {
                            scale.push(a.get_string().remove(0));	//extract scale prefix
                        }
                        if scale.is_empty() {scale.push('0');}

                        let mut power = String::new();
                        while a.get_string().ends_with(|c: char| c.is_ascii_digit()) {
                            power.insert(0, a.get_string().pop().unwrap());	//extract power suffix
                        }
                        if power.is_empty() {power.push('1');}

                        if let Some(res) = constants::get_prec(handler.working_precision, *a.get_string()) {
                            handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision, (res*Float::with_val(handler.working_precision, Integer::parse(scale).unwrap().complete()).exp10())
                                    .pow(Integer::parse(power).unwrap().complete()))));
                        }
                    },
                    1 => {	//conversion shorthand, everything is like the 0 case but twice
                        let (from, to) = a.get_string().split_once(' ').unwrap();
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
                                handler.main_stack.push(StackObject::Float(Float::with_val(handler.working_precision,
                                        (nfrom*Float::with_val(handler.working_precision, Integer::parse(kfrom).unwrap().complete()).exp10())
                                            .pow(Integer::parse(pfrom).unwrap().complete())/
                                        (nto*Float::with_val(handler.working_precision, Integer::parse(kto).unwrap().complete()).exp10())
                                            .pow(Integer::parse(pto).unwrap().complete()))));
                            }
                        }
                    },
                    _ => {
                        output.push(output!(Err, format!("! Too many spaces in constant lookup/unit conversion string \"{}\"", a.get_string())));
                    },
                }
            }
            else {	//"print" number to string
                handler.main_stack.push(StackObject::String(super::float_to_string(*a.get_float(), handler.parameter_stack.last().unwrap().2.clone(), handler.parameter_stack.last().unwrap().0.clone())));
            }
        }
    }
	output
}