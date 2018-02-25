/* Parser for XSPF files
 *
 * This is simply a wrapper around an underlying XML reading library,
 * so that we can just abstract out the bits we want to expose.
 */
extern crate sxd_document;
use self::sxd_document::parser;    // XXX: See https://stackoverflow.com/a/27653246/6531515 for why we need "self::"

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


/* ********************************************** */
/* Playlist Types */

/* A track listing in the playlist */
pub struct Track {
	/* Descriptive name assigned to this track */
	name: String,
	/* Duration (in ms) of the track - as stored in the file */
	duration: Option<i64>,
	
	/* Full name of the track itself (v<num>_<name>.<mp3/flac>) */
	filename: String,
	/* Date string of the track (i.e. parent directory) */
	date: String,
	
	/* Full path (extracted from the file) */
	path: String,
}

/* ------------------------------------------- */

/* Container for everything about the playlist */
pub struct XSPF_Playlist {
	tracks : Vec<Track>
}

/* ********************************************** */
/* Parsing API */

/* Read the file into a string, for easier processing
 *
 * FIXME: It's not nice having the entire file loaded in memory like this
 *        especially on large files. That said, most playlists should be small.
 */
fn parse_file(filename: &str) -> String
{
	let mut f = File::open(filename).expect("ERROR: File not found");
	
	let mut contents = String::new();
	f.read_to_string(&mut contents)
	 .expect("ERROR: Something went wrong reading the file");
	 
	/* Return the string. The program will have "panic()'d if anything went wrong,
	 * so this function will always just return a string
	 */
	contents
}


/* Process the XML Tree */
pub fn parse_xspf(filename: &str) -> Option<XSPF_Playlist>
{
	/* 1) Read contents of file to a string */
	let xml = parse_file(filename);
	
	/* 2) Parse the file into a DOM tree*/
	// FIXME: properly handle the parsing failures here
	let package = parser::parse(xml.as_ref()).expect("Failed to parse");
	let doc = package.as_document();
	
	/* 3) Create and return new playlist object from the DOM */
	let playlist = XSPF_Playlist::from_xml_doc(doc);
	Some(playlist)
}

/* ********************************************** */


