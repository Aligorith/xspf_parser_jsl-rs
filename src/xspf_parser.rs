/* Parser for XSPF files
 *
 * This is simply a wrapper around an underlying XML reading library,
 * so that we can just abstract out the bits we want to expose.
 */
extern crate minidom;
use self::minidom::Element;

use std::fs::File;
use std::io::prelude::*;
//use std::path::Path;


/* ********************************************** */
/* Playlist Types */

/* A track listing in the playlist */
#[derive(Debug)]
pub struct Track {
	/* Descriptive name assigned to this track */
	pub name: String,
	/* Duration (in ms) of the track - as stored in the file */
	pub duration: Option<i64>,
	
	/* Full name of the track itself (v<num>_<name>.<mp3/flac>) */
	pub filename: String,
	/* Date string of the track (i.e. parent directory) */
	pub date: String,
	
	/* Full path (extracted from the file) */
	pub path: String,
}

const FILE_URI_PREFIX: &'static str = "file:///";
const MP3_EXTN: &'static str = ".mp3";
const FLAC_EXTN: &'static str = ".flac";

impl Track {
	/* Generate a track element from a file path */
	pub fn from_filepath(path: &str) -> Result<Track, &'static str>
	{
		/* full unmodfied path */
		let fullpath = path.to_string();
		
		/* extra filename and date from the last parts of the path */
		// TODO: Sanity checking!
		let mut path_elems : Vec<&str> = fullpath.split("/").collect();
		
		let date = path_elems.pop().unwrap().to_string();
		let filename = path_elems.pop().unwrap().to_string();
		
		// FIXME: This needs some fancy regex filtering here...
		let name = if filename.ends_with(MP3_EXTN) {
		               let end_idx = filename.len() - MP3_EXTN.len() - 1;
		               filename[ .. end_idx].to_string()
		           }
		           else if filename.ends_with(FLAC_EXTN) {
		                let end_idx = filename.len() - FLAC_EXTN.len() - 1;
		                filename[ .. end_idx].to_string()
		           }
		           else {
		                filename.to_string()
		           };
		
		
		/* Construct and return a track */
		Ok(Track {
			name: name.clone(),
			duration: None,  /* Currently unknown */
			filename: filename.clone(),
			date: date.clone(),
			path: fullpath.clone()
		})
	}
	
	/* Generate a track element from a URI */
	pub fn from_uri(uri: &str) -> Result<Track, &'static str>
	{
		if uri.starts_with(FILE_URI_PREFIX) {
			// TODO: optimise this prefix stripping
			let filename = uri[FILE_URI_PREFIX.len() ..].to_string();
			Track::from_filepath(&filename)
		}
		else {
			/* Unsupported URI */
			Err("Unsupported URI - Must start with 'file:///'")
		}
	}
	
	
	/* Generate & populate track's details, given the element describing a track */
	pub fn from_xml_elem(e_track: &Element) -> Result<Track, &'static str>
	{
		let e_location = e_track.children().find(|&& ref x| x.name() == "location");
		let e_duration = e_track.children().find(|&& ref x| x.name() == "duration");
		
		if e_location.is_some() {
			let track = Track::from_uri(e_location.unwrap().text().as_ref());
			match track {
				Ok(mut t) => {
					/* Try to add duration to the track */
					if e_duration.is_some() {
						let duration_str = e_duration.unwrap().text();
						if let Ok(duration) = duration_str.parse::<i64>() {
							t.duration = Some(duration);
						}
					}
					
					/* Return track */
					Ok(t)
				},
				Err(e) => {
					/* Propagate error */
					Err(e)
				}
			}
		}
		else {
			/* No location, no use */
			Err("Element skipped as no location info found")
		}
	}
}

/* ------------------------------------------- */

/* Container for everything about the playlist */
#[derive(Debug)]
pub struct XspfPlaylist {
	pub tracks : Vec<Track>,
	pub title : Option<String>
}

impl XspfPlaylist {
	/* Generate & populate playlist, given the root element of the */
	pub fn from_xml_tree(root: Element) -> XspfPlaylist
	{
		let mut tracklist : Vec<Track> = Vec::new();
		let mut title = None;
		
		/* Go over DOM, pulling out what we need */
		for e_section in root.children() {
			match e_section.name().as_ref() {
				"title" => {
					title = Some(e_section.text());
				},
				
				"trackList" => {
					for e_track in e_section.children() {
						println!("Processing track...");
						if let Ok(track) = Track::from_xml_elem(e_track) {
							println!("   Added {0:?}", track);
							tracklist.push(track);
						}
						else {
							println!("   Error encountered");
						}
					}
					
					// e_section.children()
					//          .map(|t| Track::from_xml_elem(t))
					//          .collect();
				},
				
				_ => { /* Unhandled */ }
			}
		}
		
		/* Return playlist instance populated with this info */
		XspfPlaylist {
			tracks: tracklist,
			title: title
		}
	}
	
	/* Utility - Number of tracks in playlist */
	pub fn len(&self) -> usize
	{
		self.tracks.len()
	}
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
pub fn parse_xspf(filename: &str) -> Option<XspfPlaylist>
{
	/* 1) Read contents of file to a string */
	let xml_file = parse_file(filename);
	
	/* 2) Parse the file into a DOM tree*/
	// FIXME: properly handle the parsing failures here
	let root: Element = xml_file.parse().unwrap();
	
	/* 3) Create and return new playlist object from the DOM */
	let playlist = XspfPlaylist::from_xml_tree(root);
	Some(playlist)
}

/* ********************************************** */


