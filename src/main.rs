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
use std::process::Command;

//use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

mod track_duration;  // XXX: Have this as part of xspf_parser?
mod track_name_info; // XXX: Have this as part of xspf_parser

mod xspf_parser;

/* Aliases */
use track_name_info::TrackExtension as TrackExtension;

/* ********************************************* */

fn print_usage_info()
{
	let s = indoc!(
                  "Usage:  xspf_tools <mode> <in.xspf> [<outfile/dir>] [.sub-mode.   ...command-args...]
                  
                        where <mode> is one of the following:
                           * help      Prints this text
                           
                           * dump      Prints summary of the important identifying info gained from the playlist
                           * runtime   Prints summary of the total running time of the playlist
                           
                           * list      Writes the file paths of all tracks in the playlist to <outfile>
                           * json      Extracts the useful info out of the file, and dumps to JSON format
                                       in <outfile> for easier handling
                           
                           * copy      Copies all the files named in the playlist to the nominated folder <outdir>.
                           
                           * convert   Similar to copy, but it takes an additional <format> arg (command-args[0])
                                       specifying the output format to convert everything to. Any additional arguments
                                       after that are passed directly to FFMPEG (assuming FFMPEG is on the path).
                  "
                  );
	println!("{}", s);
	
	let program_name = env::args().nth(0).unwrap(); /* This is safe, as this is *always* in args */
	let current_dir  = env::current_dir().unwrap();
	
	println!("\n[{0:}] running from {1:?}", program_name, current_dir.display())
}

/* ********************************************* */

/* Type wrapper for these functions
 * Note: This is used instead of a simple type-def as there may be a variable number of arguments required.
 *       
 *       Doing it this way means that functions that don't need all the args can be passed to the same
 *       basic handler function.
 */
enum XspfProcessingModeFunc {
	/* Only takes an input filename - Output filename is not used / causes an error if defined */
	InOnly(fn(in_file: &str)),
	
	/* Default mode that only takes Input (in_file) and Optional Output (out_file) paths */
	InOut(fn(in_file: &str, out_file: Option<&String>)),
	
	/* InOut with additional arguments (optional) */
	InOutWithArgs(fn(in_file:&str, out_file: Option<&String>, args: &Vec<String>)),
	
	/* InOut with Mode and additional arguments */
	InOutModeWithArgs(fn(in_file: &str, out_file: &str, mode: &str, args: &Vec<String>)),
}

/* --------------------------------------------- */

/* Extract the vector of args to pass to the (sub)-command being run (e.g. FFMPEG arguments) */
fn extract_command_args_list(program_args: &Vec<String>, start_index: usize) -> Box<Vec<String>>
{
	// 0 = program name, 1 = program mode, 2 = first mode-related arg
	assert!(start_index >= 2);
	
	// Extract the remaining args from the vector
	let command_args_option = program_args.get(start_index ..);
	
	let command_args : Vec<String> =
		if let Some(command_args_slice) = command_args_option {
			command_args_slice.to_vec()
		}
		else {
			Vec::new()
		};
	
	return Box::new(command_args);
}

/* Handle the "out_file" parameter to determine if we're writing to stdout or a named file */
// FIXME: Handle errors with not being able to open the file
fn get_output_stream(out_file: Option<&String>) -> Box<dyn Write>
{
	let out_writer = match out_file {
		Some(x) => {
			let path = Path::new(x);
			Box::new(File::create(&path).unwrap()) as Box<dyn Write>
		},
		None => {
			Box::new(io::stdout()) as Box<dyn Write>
		},
	};
	out_writer
}

/* Ensure output directory exists
 * ! This function will terminate the process if the directory couldn't be created,
 *   or some other error occurs that prevents it doing its job.
 * > Returns the path object representing the root directory that was just created
 */
fn ensure_output_directory_exists(out_dir: &str) -> &Path
{
	let dst_path_root = Path::new(out_dir);
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
	dst_path_root
}

/* Write manifest of the set of files copied to <out_path>/<playlist_filename>.m3u */
fn write_copied_files_manifest(input_playlist_filename: &str, out_path: &str, dest_filenames: &Vec<String>)
{
	let playlist_filestem = Path::new(input_playlist_filename).file_stem();
	let playlist_filename = match playlist_filestem {
								Some(n) => n.to_str().unwrap(),
								None    => input_playlist_filename
							};
	let manifest_path = Path::new(out_path).join(format!("{playlist}.m3u", playlist=playlist_filename));
	println!("\nWriting manifest of copied files to {0}", manifest_path.display());
	
	match File::create(&manifest_path) {
		Ok(mut f) => {
			for filename in dest_filenames.iter() {
				match writeln!(f, "{}", filename) {
					Err(why) => {
						eprintln!("ERROR: Problem encountered while writing manifest file - {}", why);
						break;
					}
					_ => { /* keep going */ }
				}
			}
		},
		Err(why) => {
			eprintln!("ERROR: Could not write track manifest to {0:?}", manifest_path);
			eprintln!("       Reason: {:?}", why)
		}
	}
}

/* --------------------------------------------- */

/* Debug mode showing summary of most salient information about the contents of the playlist */
fn dump_output_mode(in_file: &str)
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
		let mut out : Box<dyn Write> = get_output_stream(out_file);
		
		/* Write out the full filepath for each track to separate lines in the output stream */
		for track in xspf.tracks.iter() {
			match writeln!(out, "{0}", track.path) {
				Err(why) => {
					eprintln!("ERROR: {}", why);
					break;
				},
				_ => { /* continue */}
			}
		}
	}
}


/* Extract all the relevant info from playlist, and dump it into a JSON file for further processing */
fn json_output_mode(in_file: &str, out_file: Option<&String>)
{
	println!("JSON in='{0}', out={1:?}", in_file, out_file);
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		/* Get output stream to write to */
		let mut out : Box<dyn Write> = get_output_stream(out_file);
		
		/* Serialise XSPF to a JSON string */
		// FIXME: Warn when we cannot serialise
		match serde_json::to_string_pretty(&xspf) {
			Ok(j) => {
				/* Write entire json string to output */
				match writeln!(out, "{}", j) {
					Err(why) => {
						eprintln!("ERROR: Couldn't write JSON output - {}", why);
					},
					_ => { /* continue */}
				}
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
fn total_duration_mode(in_file: &str)
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
			let _dst_path_root = ensure_output_directory_exists(out);
			
			/* Compute track index width - number of digits of padding to display before the number */
			let track_index_width = xspf.track_index_width();
			
			/* Loop over tracks copying them to the folder */
			let mut dest_filenames : Vec<String> = Vec::new();
			
			for (track_idx, track) in xspf.tracks.iter().enumerate() {
				/* Construct filename for copied file - it needs to have enough metadata to figure out what's going on */
				let dst_filename =  if track.info.track_type == track_name_info::TrackType::UnknownType {
									    /* Just use as-is, since it doesn't follow our rules */
									    format!("Track_{track_idx:0tixw$}-{fname}",
									            track_idx=track_idx + 1,
									            tixw=track_index_width,
									            fname=track.filename)
									}
									else {
									    /* Reformat the name, using the info we've learned about it */
									    format!("Track_{track_idx:0tixw$}-{date}-{tt}{index:02}_{name}.{ext:?}",
									            track_idx=track_idx + 1,
									            tixw=track_index_width,
									            date=track.date,
									            tt=track.info.track_type.shortname_safe(),
									            index=track.info.index,
									            name=track.info.name,
									            ext=track.info.extn)
									};
				
				/* Construct paths to actually perform the copying to/from */
				let src_path = &track.path;
				let dst_path = Path::new(out).join(dst_filename.to_string());
				
				/* Perform the copy operation */
				match fs::copy(src_path, dst_path) {
					Ok(_)  => {
						println!("   Copied {src} => <outdir>/{dst}", 
						         src=track.filename, dst=dst_filename);
						dest_filenames.push(dst_filename);
					},
					Err(e) => {
						eprintln!("! ERROR: Couldn't copy {src} => <ourdir>/{dst}!",
						          src=track.filename, dst=dst_filename);
						eprintln!("  Reason: {}", e);
						
						/* XXX: Should we stop instead? We don't have any other way to keep going otherwise! */
						//process::exit(1);
					}
				}
			}
			
			/* Dump list of copied files to <out_path>/<playlist_filename>.m3u
			 * (i.e. a playable playlist, that also acts as a manifest of the set of files copied)
			 */
			write_copied_files_manifest(in_file, out, &dest_filenames);
		}
	}
	else {
		eprintln!("ERROR: The third argument should specify the directory to copy the source files to");
		process::exit(1);
	}
}


/* Similar to copy, but converts all the files to the specified format using FFMPEG */
fn convert_files_mode(in_file: &str, out_path: &str, convert_mode: &str, args: &Vec<String>)
{
	println!("Convert Files infile='{0}', outdir={1:}", in_file, out_path);
	
	/* Check that FFMPEG works/is available... */
	let ffmpeg_testrun_result = Command::new("ffmpeg").arg("-version")
									.output()
									.expect("Failed to find and run ffmpeg");
	
	println!("ffmpeg test result = {}", ffmpeg_testrun_result.status);
	if !ffmpeg_testrun_result.status.success() {
		eprintln!("Aborting: ffmpeg returned abnormal status from test run");
		process::exit(1);
	}
	
	/* Determine what mode to use, and set the initial arguments for that mode */
	let mut export_format: TrackExtension = TrackExtension::Placeholder;
	let mut ffmpeg_args: Vec<String> = Vec::new();
	
	match convert_mode.parse::<TrackExtension>() {
		/* Supported Formats */
		// XXX: Only audio ones initially, since that's easier than generating visuals for those without them
		Ok(TrackExtension::mp3) => {
			export_format = TrackExtension::mp3;
			
			ffmpeg_args.push("-vn".to_string()); // Only audio feed
			//ffmpeg_args.push(format!("-acodec={:?}", export_format).to_string());
			
			// TODO: Audio quality
		},
		Ok(TrackExtension::flac) => {
			export_format = TrackExtension::flac;
			
			ffmpeg_args.push("-vn".to_string()); // Only audio feed
			//ffmpeg_args.push(format!("-acodec={:?}", export_format).to_string());
		},
		Ok(TrackExtension::ogg) => {
			export_format = TrackExtension::ogg;
			
			ffmpeg_args.push("-vn".to_string()); // Only audio feed
			//ffmpeg_args.push(format!("-acodec={:?}", export_format).to_string());
		},
		
		/* Unsupported formats - All video formats and Unknown Extensions */
		Ok(TrackExtension::Unknown(ext)) => {
			eprintln!("Error: Unsupported/unknown output format ({0:?})", ext);
			process::exit(1);
		},
		Ok(t) => {
			eprintln!("Error: Cannot export to video format ({0:?})", t);
			process::exit(1);
		},
		
		/* Parsing Error - Invalid argument */
		_ => {
			eprintln!("Error: Parsing error for convert_mode argument");
			process::exit(1);
		}
	}
	
	/* Add additional args the user specified on the command-line to also get passed along
	 * (i.e. allowing for customising the behaviour + tweaking it without recompiling)
	 */
	for arg in args {
		ffmpeg_args.push(arg.to_string());
	}
	
	/* Parse XSPF Playlist... */
	if let Some(xspf) = xspf_parser::parse_xspf(in_file) {
		/* Ensure outdir exists */
		let _dst_path_root = ensure_output_directory_exists(out_path);
		
		/* Compute track index width - number of digits of padding to display before the number */
		let track_index_width = xspf.track_index_width();
		
		/* Loop over tracks copying them to the folder */
		let mut dest_filenames : Vec<String> = Vec::new();
		
		for (track_idx, track) in xspf.tracks.iter().enumerate() {
			/* Construct filename for copied file - it needs to have enough metadata to figure out what's going on */
			let dst_filename =  if track.info.track_type == track_name_info::TrackType::UnknownType {
								    /* Just use as-is, since it doesn't follow our rules */
								    format!("Track_{track_idx:0tixw$}-{fname}.{ext:?}",
								            track_idx=track_idx + 1,
								            tixw=track_index_width,
								            fname=track.filename,
								            ext=export_format)
								}
								else {
								    /* Reformat the name, using the info we've learned about it */
								    format!("Track_{track_idx:0tixw$}-{date}-{tt}{index:02}_{name}.{ext:?}",
								            track_idx=track_idx + 1,
								            tixw=track_index_width,
								            date=track.date,
								            tt=track.info.track_type.shortname_safe(),
								            index=track.info.index,
								            name=track.info.name,
								            ext=export_format)
								};
			
			/* Construct paths to actually perform the copying to/from */
			let src_path = &track.path;
			let dst_path = Path::new(out_path).join(dst_filename.to_string());
			
			/* Add the file paths to the args to pass to FFMPEG...
			 * - Input filename needs to come first
			 * - Output filename needs to go last
			 */
			let mut ffmpeg_args_for_file: Vec<String> = Vec::new();
			
			for arg in &ffmpeg_args {
				/* Add each standard arg for this conversion operation */
				ffmpeg_args_for_file.push(arg.to_string());
			}
			
			ffmpeg_args_for_file.insert(0, "-i".to_string());
			ffmpeg_args_for_file.insert(1, src_path.as_str().to_string());
			
			ffmpeg_args_for_file.push(dst_path.to_str().unwrap().to_string());
			
			/* Invoke ffmpeg to convert this file... */
			println!("   Converting {src_path:?} -> {dst_path:?}...",
			         src_path = src_path, dst_path = dst_path);
			{
				println!("      Args = {ffmpeg_args:?}\n", ffmpeg_args = ffmpeg_args_for_file); // debug only
			}
			
			let ffmpeg_convert_command
				= Command::new("ffmpeg")
					.args(ffmpeg_args_for_file)
					//.output()
					.status()
					.expect("Failed to find and run ffmpeg");
					
			if ffmpeg_convert_command.success() {
				println!("     Success for {dst_path:?}\n\n",
				         dst_path = dst_path);
				dest_filenames.push(dst_filename);
			}
			else {
				eprintln!("     ERROR: Conversion failed for {src_path:?} -> {dst_path:?}!\n\n",
				          src_path = src_path, dst_path = dst_path);
				/* Don't abort... try to carry on... */
			}
		}
		
		/* Dump list of copied files to <out_path>/<playlist_filename>.m3u
		 * (i.e. a playable playlist, that also acts as a manifest of the set of files copied)
		 */
		write_copied_files_manifest(in_file, out_path, &dest_filenames);
	}
}


/* --------------------------------------------- */

fn handle_xspf_processing_mode(args: &Vec<String>, processing_func: XspfProcessingModeFunc)
{
	let in_file_option = args.get(2);
	let out_file_option = args.get(3);
	
	match in_file_option {
		Some(in_file) => {
			if in_file.ends_with(".xspf") == false {
				println!("WARNING: Input file should have the '.xspf' extension");
			}
			
			match processing_func {
				XspfProcessingModeFunc::InOnly(func) => {
					/* Input File Only. Warn if out_file is provided */
					if let Some(out_file) = out_file_option {
						eprintln!("Warning: 'output_file' argument ({out}) not required for this function",
						          out=out_file);
					}
					func(in_file);
				},
				XspfProcessingModeFunc::InOut(func) => {
					/* Input File + Optional Output File */
					func(in_file, out_file_option);
				},
				XspfProcessingModeFunc::InOutWithArgs(func) => {
					/* Input File + Optional Output File + Optional args  */
					let command_args = extract_command_args_list(args, 4);
					
					/* Run the command */
					func(in_file, out_file_option, &command_args);
				},
				XspfProcessingModeFunc::InOutModeWithArgs(func) => {
					/* Input File + Mandatory Output File/Directory + Mandatory Mode + Optional Args */
					let out_path = out_file_option.expect("Output file/directory must be supplied as the 3rd argument to the program");
					let mode_arg = args.get(4).expect("Mode argument must be supplied as the 4th argument to the program");
					
					let command_args = extract_command_args_list(args, 5);
					
					/* Run the command */
					func(in_file, out_path, mode_arg, &command_args);
				}
			}
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
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOnly(dump_output_mode));
			},
			
			"list" => {
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOut(list_output_mode));
			},
			
			"json" => {
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOut(json_output_mode));
			},
			
			"runtime" => {
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOnly(total_duration_mode));
			},
			
			"copy" => {
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOut(copy_files_mode));
			},
			
			"convert" => {
				handle_xspf_processing_mode(&args, XspfProcessingModeFunc::InOutModeWithArgs(convert_files_mode));
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
