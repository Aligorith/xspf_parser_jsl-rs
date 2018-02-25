#[macro_use] extern crate indoc;

use std::env;

//mod xspf_parser;
#[macro_use] mod logic_macros;

fn print_usage_info()
{
	let s = indoc!(
                  "Usage:  xspf_tools <mode> <in.xspf> [<outfile>]
                  
                        where <mode> is one of the following:
                           * help    Prints this text
                           * list    Writes the filenames of all tracks in the playlist to <outfile>
                           * json    Extracts the useful info out of the file, and dumps to JSON format
                                     in <outfile> for easier handling
                  "
                  );
	println!("{}", s);
}

fn list_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("List in='{0}', out={1:?}", in_file, out_file);
}

fn json_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("JSON in='{0}', out={1:?}", in_file, out_file);
}

fn main()
{
	let args: Vec<String> = env::args().collect();
	
	if (args.len() > 2) && (&args[1] != "help") {
		let in_file = &args[2];
		let out_file = if args.len() == 4 { Some(&args[3]) } else { None };
		
		match args[1].as_ref() {
			"list" => {
				list_output_mode(&in_file, out_file);
			},
			"json" => {
				json_output_mode(&in_file, out_file);
			},
			arg => {
				println!("Unrecognised option: '{0}'", arg);
				print_usage_info();
			}
		}
	}
	else if (args.len() > 1) && elem!(&args[1], "list", "json") {
		println!("ERROR: You need to supply a .xspf filename as the second argument!\n");
		print_usage_info();
	}
	else {
		print_usage_info();
	}
}
