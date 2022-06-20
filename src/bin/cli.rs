use std::io::{stdin, stdout, Write};

use dcim::handler;

const HELPMSG: &str = "
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

const PRECISION: u32 = 256;

fn main () {
    let args: Vec<String> = std::env::args().collect();
    let mut args: Vec<&str> = args.iter().map(|s| &**s).collect(); //convert to &str
	args.remove(0);	//remove name of executable

    if args.is_empty() {
        args = vec!["-i"];
    }
    let mode = args.remove(0);

    let mut handler = handler::Handler::new(PRECISION);

    match mode {
        "--interactive"|"-i"|"i" => {
            //ability to force interactive mode just in case
            
            interactive_mode(handler);
        },
        "--expression"|"-e"|"e" => {
           
            let output = handler.handle(handler::Input::Expression(args));
            manage_output(&output, &mut handler);

            
        },
        "--file"|"-f"|"f" => {
            let output = handler.handle(handler::Input::File(args));
            manage_output(&output, &mut handler);
        },
        "--help"|"-h"|"h" => {
            println!("{}", HELPMSG);
        },
        _ => {
            eprintln!("! Invalid option \"{}\", use h for option syntax help", mode);
        },
    }
}

fn manage_output(output: &Vec<handler::Output>, handler: &mut handler::Handler) {
    output.iter().for_each(|output| {
        let message = match output {
            Ok((message, _)) => message,
            Err(message) => message,
        };
        println!("{}", message);

        match 
            match output {
            Ok((_, command)) => command,
            Err(message) => panic!("{}", message),
            } 
        {
            handler::Command::Exit => {
                std::process::exit(0);
            },
            handler::Command::Interactive => {
                interactive_mode(handler.clone());
            },
            _ => {
                eprintln!("! WIP Not implemented yet!");
            }
        }
    });
}

fn interactive_mode(mut handler: handler::Handler) {
    loop {
        //prompt for user input
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("! Unable to read standard input: {}", error);
                break;
            }
        }
        if input.is_empty() {
            print!("\r");
            break;
        }
        let output = handler.handle(handler::Input::Interactive(&input));
        manage_output(&output, &mut handler);
    }
}