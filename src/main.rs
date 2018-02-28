/* Main entrypoint for "xspf_tools" executable AND 
 * also the implicit crate that it and all its stuff lives in
 */

/* macro_use defines need to happen in the crate root - https://stackoverflow.com/a/39175997/6531515 */
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate indoc;
#[macro_use] mod logic_macros;

use std::env;

use std::fs::File;
use std::path::Path;
use std::io::{self, Write};

mod xspf_parser;

/* ********************************************* */

fn print_usage_info()
{
	let s = indoc!(
                  "Usage:  xspf_tools <mode> <in.xspf> [<outfile>]
                  
                        where <mode> is one of the following:
                           * help    Prints this text
                           * dump    Prints summary of the important identifying info gained from the playlist
                           * list    Writes the file paths of all tracks in the playlist to <outfile>
                           * json    Extracts the useful info out of the file, and dumps to JSON format
                                     in <outfile> for easier handling
                  "
                  );
	println!("{}", s);
}

/* ********************************************* */

type XspfProcessingModeFunc = fn(in_file: &str, out_file: Option<&String>);

/* --------------------------------------------- */

/* Handle the "out_file" parameter to determine if we're writing to stdout or a named file */
// FIXME: Handle errors with not being able to open the file
fn get_output_stream(out_file: Option<&String>) -> Box<Write>
{
	let out_writer = match out_file {
		Some(x) => {
			let path = Path::new(x);
			Box::new(File::create(&path).unwrap()) as Box<Write>
		},
		None => {
			Box::new(io::stdout()) as Box<Write>
		},
	};
	out_writer
}

/* --------------------------------------------- */

/* NOTE: "out_file" is unused/unneeded, hence the underscore */
fn dump_output_mode(in_file: &str, _out_file: Option<&String>)
{
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		println!("{0} Tracks:", xspf.len());
		for (i, track) in xspf.tracks.iter().enumerate() {
			println!("  {0} | filename = '{1}', date = {2}, duration = {3:?}",
			         i, track.filename, track.date, track.duration);
			println!("        Info: {:?}", track.info);
		}
	}
}

fn list_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("List in='{0}', out={1:?}", in_file, out_file);
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		/* Get output stream to write to */
		let mut out : Box<Write> = get_output_stream(out_file);
		
		/* Write out the full filepath for each track to separate lines in the output stream */
		for track in xspf.tracks.iter() {
			// FIXME: How do we handle the Result<> here?
			writeln!(out, "{0}", track.path);
		}
	}
}


fn json_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("JSON in='{0}', out={1:?}", in_file, out_file);
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		// TODO: Serialise playlist to file
	}
}

/* --------------------------------------------- */

fn handle_xspf_processing_mode(args: &Vec<String>, processing_func: XspfProcessingModeFunc)
{
	let in_file = args.get(2);
	let out_file = args.get(3);
	
	match in_file {
		Some(f) => {
			if f.ends_with(".xspf") == false {
				println!("WARNING: Input file should have the '.xspf' extension");
			}
			
			processing_func(f, out_file);
		},
		None => {
			println!("ERROR: You need to supply a .xspf filename as the second argument\n");
			print_usage_info();
		}
	}
}


/* ********************************************* */

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
			"dump" => {
				handle_xspf_processing_mode(&args, dump_output_mode);
			},
			
			"list" => {
				handle_xspf_processing_mode(&args, list_output_mode);
			},
			
			"json" => {
				handle_xspf_processing_mode(&args, json_output_mode);
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
