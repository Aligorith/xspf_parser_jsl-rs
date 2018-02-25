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
	
	if let Some(mode) = args.get(1) {
		/* A mode string was supplied - Process it!
		 *
		 * XXX: It would've been nice to handle the unsupplied case here too,
		 *      but then, we wouldn't be able to do the mode.as_ref() thing
		 *      that's needed to make string-case matching work
		 *      (i.e. otherwise we get type errors about "std::string::String vs str")
		 */
		match mode.as_ref() {
			"list" => {
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
			
			"json" => {
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
			
			"help" => {
				print_usage_info();
			},
			
			arg => {
				println!("Unrecognised option: '{0:?}'", arg);
				print_usage_info();
			}
		}
	}
	else {
		/* No mode arg at all - i.e. user really doesn't know what they're doing */
		/* XXX: ideally, this would have been included above, instead of in here... */
		print_usage_info();
	}
}
