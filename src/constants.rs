use rug::{Integer, integer::Order, Complete, Float, float::{Round, Constant}, ops::Pow};
use std::time::{SystemTime, Duration};

use crate::handler::{Handler, Output, StackObject, Command, output};
use crate::commands;

pub mod env {
	use rug::Integer;
	use std::cmp::Ordering;
    pub const INT_ROUND_DEFAULT: (Integer, Ordering) = (Integer::ZERO, Ordering::Equal);

	use std::sync::Arc;
	use rug::rand::RandState;
	use lazy_static::lazy_static;
	lazy_static! {
    pub static ref RNG: Arc<RandState<'static>> = Arc::new(RandState::new());
	}
}


//library of constants and unit conversion factors
//unless specified, unit factors are based on the most prevalent international standard units for their respective quantities
//ex: "in" (inch) returns 0.0254, thus executing 20[in]"* converts 20 inches to meters (0.508)
pub fn get_prec(prec: u32, key: String) -> Option<Float> {
	type iDef = i64;
	match key.as_str() {
		/*----------------------------
			MATHEMATICAL CONSTANTS
		----------------------------*/
		"e" => {Some(Float::with_val(prec, 1 as iDef).exp())}
		"pi" => {Some(Float::with_val(prec, Constant::Pi))}
		"gamma" => {Some(Float::with_val(prec, Constant::Euler))}
		"phi" => {Some((Float::with_val(prec, 5 as iDef).sqrt()+1 as iDef)/2 as iDef)}
		"deg"|"°" => {Some(Float::with_val(prec, Constant::Pi)/180 as iDef)}
		"gon"|"grad" => {Some(Float::with_val(prec, Constant::Pi)/200 as iDef)}
		/*------------------------
			PHYSICAL CONSTANTS
		------------------------*/
		"c" => {Some(Float::with_val(prec, 299792458 as iDef))}
		"hbar" => {Some(sci_to_flt(prec, 662607015, -42).unwrap()/(2 as iDef*Float::with_val(prec, Constant::Pi)))}
		"G" => {sci_to_flt(prec, 6674, -3)}
		"qe" => {sci_to_flt(prec, 1602176634, -28)}
		"NA" => {sci_to_flt(prec, 602214076, 31)}
		"kB" => {sci_to_flt(prec, 1380649, -29)}
		"u" => {sci_to_flt(prec, 1660539066, -36)}
		"lp" => {sci_to_flt(prec, 16162, -39)}
		"tp" => {sci_to_flt(prec, 5391, -47)}
		"mp" => {sci_to_flt(prec, 21764, -12)}
		"Tp" => {sci_to_flt(prec, 14167, 28)}
		/*------------------
			LENGTH UNITS
		------------------*/
		"in" => {sci_to_flt(prec, 254, -4)}
		"ft" => {Some(get_prec(prec, "in".to_string()).unwrap()*12 as iDef)}
		"yd" => {Some(get_prec(prec, "ft".to_string()).unwrap()*3 as iDef)}
		"m" => {Some(Float::with_val(prec, 1 as iDef))}
		"fur" => {Some(get_prec(prec, "ft".to_string()).unwrap()*660 as iDef)}
		"mi" => {Some(get_prec(prec, "ft".to_string()).unwrap()*5280 as iDef)}
		"nmi" => {Some(Float::with_val(prec, 1852 as iDef))}
		"AU" => {Some(Float::with_val(prec, 149597870700i64))}
		"ly" => {Some(Float::with_val(prec, 9460730472580800i64))}
		"pc" => {Some(Float::with_val(prec, 96939420213600000i64)/Float::with_val(prec, Constant::Pi))}
		/*-------------------------------
			   AREA & VOLUME UNITS
			with no length equivalent
		-------------------------------*/
		"ac"|"acre" => {sci_to_flt(prec, 40468564224, -7)}
		"l" => {Some(Float::with_val(prec, 10 as iDef).pow(-3 as iDef))}
		"ifloz" => {sci_to_flt(prec, 284130625, -13)}
		"ipt" => {Some(get_prec(prec, "ifloz".to_string()).unwrap()*20 as iDef)}
		"iqt" => {Some(get_prec(prec, "ifloz".to_string()).unwrap()*40 as iDef)}
		"igal" => {Some(get_prec(prec, "ifloz".to_string()).unwrap()*160 as iDef)}
		"ibu"|"ibsh" => {Some(get_prec(prec, "ifloz".to_string()).unwrap()*1280 as iDef)}
		"ufldr" => {sci_to_flt(prec, 36966911953125, -19)}
		"tsp" => {Some(get_prec(prec, "ufldr".to_string()).unwrap()/3 as iDef*4 as iDef)}
		"tbsp" => {Some(get_prec(prec, "ufldr".to_string()).unwrap()*4 as iDef)}
		"ufloz" => {Some(get_prec(prec, "ufldr".to_string()).unwrap()*8 as iDef)}
		"upt" => {Some(get_prec(prec, "ufloz".to_string()).unwrap()*16 as iDef)}
		"uqt" => {Some(get_prec(prec, "ufloz".to_string()).unwrap()*32 as iDef)}
		"ugal" => {Some(get_prec(prec, "ufloz".to_string()).unwrap()*128 as iDef)}
		"bbl" => {Some(get_prec(prec, "ugal".to_string()).unwrap()*42 as iDef)}
		"udpt" => {sci_to_flt(prec, 5506104713575, -16)}
		"udqt" => {Some(get_prec(prec, "udpt".to_string()).unwrap()*2 as iDef)}
		"udgal" => {Some(get_prec(prec, "udpt".to_string()).unwrap()*8 as iDef)}
		"ubu"|"ubsh" => {Some(get_prec(prec, "udpt".to_string()).unwrap()*64 as iDef)}
		"dbbl" => {sci_to_flt(prec, 115627123584, -12)}
		/*----------------
			MASS UNITS
		----------------*/
		"ct" => {sci_to_flt(prec, 2, -4)}
		"oz" => {sci_to_flt(prec, 28349523125, -12)}
		"lb" => {Some(get_prec(prec, "oz".to_string()).unwrap()*16 as iDef)}
		"kg" => {Some(Float::with_val(prec, 1 as iDef))}
		"st" => {Some(get_prec(prec, "lb".to_string()).unwrap()*14 as iDef)}
		"t" => {Some(get_prec(prec, "lb".to_string()).unwrap()*2240 as iDef)}
		/*----------------
			TIME UNITS
		----------------*/
		"s" => {Some(Float::with_val(prec, 1 as iDef))}
		"min" => {Some(Float::with_val(prec, 60 as iDef))}
		"h" => {Some(get_prec(prec, "min".to_string()).unwrap()*60 as iDef)}
		"d" => {Some(get_prec(prec, "h".to_string()).unwrap()*24 as iDef)}
		"w" => {Some(get_prec(prec, "d".to_string()).unwrap()*7 as iDef)}
		/*-----------------
			OTHER UNITS
		-----------------*/
		"J" => {Some(Float::with_val(prec, 1 as iDef))}
		"cal" => {sci_to_flt(prec, 4184, -3)}
		"Pa" => {Some(Float::with_val(prec, 1 as iDef))}
		"atm" => {Some(Float::with_val(prec, 101325 as iDef))}
		"psi" => {sci_to_flt(prec, 6894757293168, -9)}
		/*------------------------------
			SPECIAL VALUES/FUNCTIONS
		------------------------------*//*
		"time" => {Some(Float::with_val(prec, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs()))}
		"timens" => {Some(Float::with_val(prec, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_nanos()))}
		"pid" => {Some(Float::with_val(prec, std::process::id()))}
		"abort" => {std::process::abort();}
		"crash" => {get_prec(prec, "crash".to_string())}	//stack overflow through recursion
		"panic" => {std::panic::panic_any(
			unsafe {if let Some(ptr) = MSTK.last() {
				if ptr.t {&ptr.s} else {"Manual panic"}}
			else {"Manual panic"}});}
		"author" => {Some(Float::with_val(prec, 43615 as iDef))}	//why not
		*/
		_ => {
			eprintln!("! Constant/conversion factor \"{}\" doesn't exist", key);
			None
		}
		
	}
}

//scientific notation to Some(Float), for brevity
fn sci_to_flt(prec: u32, man: i128, exp: i128) -> Option<Float> {
	Some(Float::with_val(prec, man)*Float::with_val(prec, exp).exp10())
}