#[macro_use] extern crate indoc;

use std::env;


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

fn main()
{
	let args: Vec<String> = env::args().collect();
	
	if (args.len() > 2) && (&args[1] != "help") {
		let in_file = &args[2];
		let out_file = if args.len() == 4 { Some(&args[3]) } else { None };
		
		match args[1].as_ref() {
			"list" => {
				println!("List in='{0}', out={1:?}", in_file, out_file);
			},
			"json" => {
				println!("JSON in='{0}', out={1:?}", in_file, out_file);
			},
			arg => {
				println!("Unrecognised option: '{0}'", arg);
				print_usage_info();
			}
		}
	}
	else {
		print_usage_info();
	}
}
