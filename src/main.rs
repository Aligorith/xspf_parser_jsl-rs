/* Main entrypoint for "xspf_tools" executable AND 
 * also the implicit crate that it and all its stuff lives in
 */

/* macro_use defines need to happen in the crate root - https://stackoverflow.com/a/39175997/6531515 */
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate indoc;
#[macro_use] extern crate serde_derive;
#[macro_use] mod logic_macros;

extern crate serde;
extern crate serde_json;

//use serde_json::Error;

use std::env;
use std::process;

use std::fs::{self, File};
use std::path::Path;
use std::io::{self, Write};

mod xspf_parser;

/* ********************************************* */

fn print_usage_info()
{
	let s = indoc!(
                  "Usage:  xspf_tools <mode> <in.xspf> [<outfile/dir>]
                  
                        where <mode> is one of the following:
                           * help      Prints this text
                           
                           * dump      Prints summary of the important identifying info gained from the playlist
                           * runtime   Prints summary of the total running time of the playlist
                           
                           * list      Writes the file paths of all tracks in the playlist to <outfile>
                           * json      Extracts the useful info out of the file, and dumps to JSON format
                                       in <outfile> for easier handling
                           
                           * copy      Copies all the files named in the playlist to the nominated folder <outdir>
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

/* Debug mode showing summary of most salient information about the contents of the playlist */
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


/* Extract filenames for all tracks from the playlist */
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


/* Extract all the relevant info from playlist, and dump it into a JSON file for further processing */
fn json_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("JSON in='{0}', out={1:?}", in_file, out_file);
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		/* Get output stream to write to */
		let mut out : Box<Write> = get_output_stream(out_file);
		
		/* Serialise XSPF to a JSON string */
		// FIXME: Warn when we cannot serialise
		match serde_json::to_string_pretty(&xspf) {
			Ok(j) => {
				/* Write entire json string to output */
				// FIXME: How do we handle the Result<> here?
				writeln!(out, "{}", j);
			},
			
			// FIXME: handle specific cases?
			Err(e) => {
				eprintln!("Couldn't convert to playlist data to JSON - {:?}", e);
				process::exit(1);
			}
		}
	}
}


/* Compute and display summary of total playing time of playlist */
/* NOTE: out_file is unneeded, as there's nothing worth writing to a file */
// TODO: Warn if outfile is provided, indicating that it'll be ignored
fn total_duration_mode(in_file: &str, _out_file: Option<&String>)
{
	println!("Total Duration Summary:");
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		/* Compute duration */
		let result = xspf.total_duration();
		
		println!("    Total Duration:  {:?} (mm:ss)", result.duration);
		println!("    Num Tracks:      {}", xspf.len());
		// TODO: include an average length estimate?
		
		if result.uncounted > 0 {
			println!("");
			println!("    Skipped Tracks:  {}", result.uncounted);
			println!("                     (Tracks may skipped if no duration data was found in the playlist)");
		}
	}
}


/* Copy all files listed in playlist to a single folder */
fn copy_files_mode(in_file: &str, out_path: Option<&String>)
{
	if let Some(out) = out_path {
		println!("Copy Files infile='{0}', outdir={1:?}", in_file, out_path);
		if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
			/* Ensure outdir exists */
			let dst_path_root = Path::new(out);
			if !dst_path_root.exists() {
				match fs::create_dir(dst_path_root) {
					Ok(_) => {
						println!("   Created new destination folder - {0:?}\n",
						         dst_path_root.canonicalize().unwrap());  // XXX: how could this go wrong?
					}
					Err(e) => {
						eprintln!("   Could not create destination folder - {0:?}",
						          dst_path_root.canonicalize().unwrap()); // XXX: how could this go wrong?
						eprintln!("   {:?}", e);
						
						/* There's no way we can recover from this */
						process::exit(1);
					}
				}
			}
			
			/* Compute track index width - number of digits of padding to display before the number */
			let track_index_width = match xspf.len() {
										0   ... 99   => 2,
										100 ... 999  => 3, /* just in case */
										_            => 4  /* it's unlikely we need more */
									};
			
			/* Loop over tracks copying them to the folder */
			for (track_idx, track) in xspf.tracks.iter().enumerate() {
				/* Construct filename for copied file - it needs to have enough metadata to figure out what's going on */
				let dst_filename = format!("Track{track_idx:0tixw$}-{date}-{tt}{index:02}_{name}.{ext:?}",
				                           track_idx=track_idx,
				                           tixw=track_index_width,
				                           date=track.date,
				                           tt=track.info.track_type.shortname_safe(),
				                           index=track.info.index,
				                           name=track.info.name,
				                           ext=track.info.extn);
				
				/* Construct paths to actually perform the copying to/from */
				let src_path = &track.path;
				let dst_path = Path::new(out).join(dst_filename.to_string());
				
				/* perform the copy */
				match fs::copy(src_path, dst_path) {
					Ok(_)  => {
						println!("   Copied {src} => <outdir>/{dst}", 
						         src=track.filename, dst=dst_filename);
					},
					Err(e) => {
						eprintln!("  ERROR: Couldn't copy {src} => <ourdir>/{dst}!",
						          src=track.filename, dst=dst_filename);
						eprintln!("  {:?}", e);
						
						/* XXX: Should we stop instead? We don't have any other way to keep going otherwise! */
						//process::exit(1);
					}
				}
			}
		}
	}
	else {
		eprintln!("ERROR: The third argument should specify the directory to copy the source files to");
		process::exit(1);
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
			},
			
			"runtime" => {
				handle_xspf_processing_mode(&args, total_duration_mode);
			},
			
			"copy" => {
				handle_xspf_processing_mode(&args, copy_files_mode);
			},
			
			"help" => {
				print_usage_info();
			},
			
			arg => {
				println!("Unrecognised option: '{0:?}'", arg);
				print_usage_info();
			},
		}
	}
	else {
		/* No mode arg at all - i.e. user really doesn't know what they're doing */
		/* XXX: ideally, this would have been included above, instead of in here... */
		print_usage_info();
	}
}
