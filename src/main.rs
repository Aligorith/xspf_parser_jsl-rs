#[macro_use] extern crate indoc;
#[macro_use] mod logic_macros;

use std::env;

//mod xspf_parser;

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
	//let xspf = xspf_parser::parse_xspf(in_file); 
}

fn json_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("JSON in='{0}', out={1:?}", in_file, out_file);
}

fn main()
{
	let args: Vec<String> = env::args().collect();
	
	match args.get(1) {
		Some("list") => {
			let in_file = args.get(2);
			let out_file = args.get(3);
			
			match in_file {
				Some(f) => {
					list_output_mode(f, out_file);
				},
				None => {
					println!("ERROR: You need to supply a .xspf filename as the second argument!\n");
					print_usage_info();
				}
			}
		},
		
		Some("json") => {
			let in_file = args.get(2);
			let out_file = args.get(3);
			
			match in_file {
				Some(f) => {
					json_output_mode(f, out_file);
				},
				None => {
					println!("ERROR: You need to supply a .xspf filename as the second argument!\n");
					print_usage_info();
				}
			}
		}
		
		Some("help") | None => {
			print_usage_info();
		},
		
		Some(arg) => {
			println!("Unrecognised option: '{0:?}'", arg);
			print_usage_info();
		}
	}
}
