pub const HELPMSG: &str = "
╭─────────────────────────╮
│   ╷           •         │
│   │                     │
│ ╭─┤  ╭─╴  •  ╶┤   ┌─┬─╮ │
│ │ │  │        │   │ │ │ │
│ ╰─┘  ╰─╴  •  ╶┴╴  ╵   ╵ │
╰─────────────────────────╯

dc improved - Feature-added rewrite of an RPN calculator/stack machine language from 1970-72
Most basic GNU dc features are unaltered, full documentation at https://github.com/43615/dcim

Options and syntax:

<nothing> | --interactive | -i | i
	Interactive mode, standard prompt loop.

(--expression | -e | e) expr1 expr2 expr3 ... [?]
	Expression mode, executes expressions in order. If the last argument is '?', enters interactive mode after expressions are done.

(--file | -f | f) file1 file2 file3 ... [?]
	File mode, executes contents of files in order. '?' behaves the same as with -e.

--help | -h | h
	Print this help message.
";

//library of constants and unit conversion factors
//unless specified, unit factors are based on the most prevalent international standard units for their respective quantities
//ex: "in" (inch) returns 0.0254, thus executing 20[in]"* converts 20 inches to meters (0.508)
pub fn get_prec(prec: u32, key: String) -> Option<Float> {
	match key.as_str() {
		/*----------------------------
			MATHEMATICAL CONSTANTS
		----------------------------*/
		"e" => {Some(Float::with_val(prec, 1).exp())}
		"pi" => {Some(Float::with_val(prec, Constant::Pi))}
		"gamma" => {Some(Float::with_val(prec, Constant::Euler))}
		"phi" => {Some((Float::with_val(prec, 5).sqrt()+1)/2)}
		"deg"|"°" => {Some(Float::with_val(prec, Constant::Pi)/180)}
		"gon"|"grad" => {Some(Float::with_val(prec, Constant::Pi)/200)}
		/*------------------------
			PHYSICAL CONSTANTS
		------------------------*/
		"c" => {Some(Float::with_val(prec, 299792458))}
		"hbar" => {Some(sci_to_flt(prec, 662607015, -42).unwrap()/(2*Float::with_val(prec, Constant::Pi)))}
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
		"ft" => {Some(constants(prec, "in".to_string()).unwrap()*12)}
		"yd" => {Some(constants(prec, "ft".to_string()).unwrap()*3)}
		"m" => {Some(Float::with_val(prec, 1))}
		"fur" => {Some(constants(prec, "ft".to_string()).unwrap()*660)}
		"mi" => {Some(constants(prec, "ft".to_string()).unwrap()*5280)}
		"nmi" => {Some(Float::with_val(prec, 1852))}
		"AU" => {Some(Float::with_val(prec, 149597870700i64))}
		"ly" => {Some(Float::with_val(prec, 9460730472580800i64))}
		"pc" => {Some(Float::with_val(prec, 96939420213600000i64)/Float::with_val(prec, Constant::Pi))}
		/*-------------------------------
			   AREA & VOLUME UNITS
			with no length equivalent
		-------------------------------*/
		"ac"|"acre" => {sci_to_flt(prec, 40468564224, -7)}
		"l" => {Some(Float::with_val(prec, 10).pow(-3))}
		"ifloz" => {sci_to_flt(prec, 284130625, -13)}
		"ipt" => {Some(constants(prec, "ifloz".to_string()).unwrap()*20)}
		"iqt" => {Some(constants(prec, "ifloz".to_string()).unwrap()*40)}
		"igal" => {Some(constants(prec, "ifloz".to_string()).unwrap()*160)}
		"ibu"|"ibsh" => {Some(constants(prec, "ifloz".to_string()).unwrap()*1280)}
		"ufldr" => {sci_to_flt(prec, 36966911953125, -19)}
		"tsp" => {Some(constants(prec, "ufldr".to_string()).unwrap()/3*4)}
		"tbsp" => {Some(constants(prec, "ufldr".to_string()).unwrap()*4)}
		"ufloz" => {Some(constants(prec, "ufldr".to_string()).unwrap()*8)}
		"upt" => {Some(constants(prec, "ufloz".to_string()).unwrap()*16)}
		"uqt" => {Some(constants(prec, "ufloz".to_string()).unwrap()*32)}
		"ugal" => {Some(constants(prec, "ufloz".to_string()).unwrap()*128)}
		"bbl" => {Some(constants(prec, "ugal".to_string()).unwrap()*42)}
		"udpt" => {sci_to_flt(prec, 5506104713575, -16)}
		"udqt" => {Some(constants(prec, "udpt".to_string()).unwrap()*2)}
		"udgal" => {Some(constants(prec, "udpt".to_string()).unwrap()*8)}
		"ubu"|"ubsh" => {Some(constants(prec, "udpt".to_string()).unwrap()*64)}
		"dbbl" => {sci_to_flt(prec, 115627123584, -12)}
		/*----------------
			MASS UNITS
		----------------*/
		"ct" => {sci_to_flt(prec, 2, -4)}
		"oz" => {sci_to_flt(prec, 28349523125, -12)}
		"lb" => {Some(constants(prec, "oz".to_string()).unwrap()*16)}
		"kg" => {Some(Float::with_val(prec, 1))}
		"st" => {Some(constants(prec, "lb".to_string()).unwrap()*14)}
		"t" => {Some(constants(prec, "lb".to_string()).unwrap()*2240)}
		/*----------------
			TIME UNITS
		----------------*/
		"s" => {Some(Float::with_val(prec, 1))}
		"min" => {Some(Float::with_val(prec, 60))}
		"h" => {Some(constants(prec, "min".to_string()).unwrap()*60)}
		"d" => {Some(constants(prec, "h".to_string()).unwrap()*24)}
		"w" => {Some(constants(prec, "d".to_string()).unwrap()*7)}
		/*-----------------
			OTHER UNITS
		-----------------*/
		"J" => {Some(Float::with_val(prec, 1))}
		"cal" => {sci_to_flt(prec, 4184, -3)}
		"Pa" => {Some(Float::with_val(prec, 1))}
		"atm" => {Some(Float::with_val(prec, 101325))}
		"psi" => {sci_to_flt(prec, 6894757293168, -9)}
		/*------------------------------
			SPECIAL VALUES/FUNCTIONS
		------------------------------*/
		"time" => {Some(Float::with_val(prec, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs()))}
		"timens" => {Some(Float::with_val(prec, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_nanos()))}
		"pid" => {Some(Float::with_val(prec, std::process::id()))}
		"abort" => {std::process::abort();}
		"crash" => {constants(prec, "crash".to_string())}	//stack overflow through recursion
		"panic" => {std::panic::panic_any(
			unsafe {if let Some(ptr) = MSTK.last() {
				if ptr.t {&ptr.s} else {"Manual panic"}}
			else {"Manual panic"}});}
		"author" => {Some(Float::with_val(prec, 43615))}	//why not
		_ => {
			eprintln!("! Constant/conversion factor \"{}\" doesn't exist", key);
			None
		}
	}
}