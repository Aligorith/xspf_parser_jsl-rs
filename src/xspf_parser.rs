/* Parser for XSPF files
 *
 * This is simply a wrapper around an underlying XML reading library,
 * so that we can just abstract out the bits we want to expose.
 */
extern crate minidom;
use self::minidom::Element;

use std::fs::File;
use std::io::prelude::*;

use track_duration::TrackDuration;
use track_name_info::FilenameInfoComponents;

/* ********************************************** */
/* Playlist Types */

/* A track listing in the playlist */
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Track {
	/* Full path (extracted from the file) */
	pub path: String,
	
	/* Full name of the track itself (v<num>_<name>.<mp3/flac>) */
	pub filename: String,
	/* Date string of the track (i.e. parent directory) */
	pub date: String,
	
	/* Duration (in ms) of the track - as stored in the file */
	pub duration: Option<TrackDuration>,
	
	/* FileInfo */
	pub info : FilenameInfoComponents
}

const FILE_URI_PREFIX: &'static str = "file:///";

impl Track {
	/* Generate a track element from a file path */
	pub fn from_filepath(path: &str) -> Result<Track, &'static str>
	{
		/* full "unmodfied" path (with the symbols replaced, so that we can find the files) */
		// TODO: Replace these hardcoded cases for something based on an encoding library (e.g. encoding_rs)
		let fullpath = path.to_string()
		                   .replace("%20", " ")
		                   .replace("%21", "!")
		                   .replace("%26", "&")
		                   .replace("%27", "'")
		                   .replace("%28", "(")
		                   .replace("%29", ")")
		                   .replace("%5B", "[")
		                   .replace("%5D", "]")
		                   .replace("%2C", ",")
		                   .replace("%C3%A8", "è")
		                   .replace("%C3%A9", "é")
		                   .replace("%C3%AD", "í")
		                   .replace("%C3%BA", "ú")
		                   .replace("%E2%80%9C", "“")
		                   .replace("%E2%80%9D", "”")
		                   .replace("%E2%80%99", "’")
		                   ;
		
		/* extra filename and date from the last parts of the path 
		 * WARNING: We're extracting these in reverse order! So first filename, then date!
		 */
		// TODO: Sanity checking!
		let mut path_elems : Vec<&str> = fullpath.split("/").collect();
		
		let filename = path_elems.pop().unwrap().to_string();
		let date = path_elems.pop().unwrap().to_string();
		
		/* Construct and return a track */
		Ok(Track {
			path: fullpath.clone(),
			filename: filename.clone(),
			date: date.clone(),
			duration: None,  /* Currently unknown */
			info: FilenameInfoComponents::new(filename.as_ref()),
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
							t.duration = Some(TrackDuration(duration));
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
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct XspfPlaylist {
	pub tracks : Vec<Track>,
	pub title : Option<String>
}

/* Helper for XspfPlaylist.total_duration() */
#[derive(Debug)]
pub struct XspfDurationTallyResult {
	pub duration : TrackDuration,      /* Total duration of tracks in this playlist */
	pub uncounted : usize              /* Number of tracks that couldn't be counted (i.e. missing durations) */
}

/* API for XspfPlaylist */
impl XspfPlaylist {
	/* Generate & populate playlist, given the root element of the */
	pub fn from_xml_tree(root: Element, filename: &str) -> XspfPlaylist
	{
		let mut tracklist : Vec<Track> = Vec::new();
		let mut title = None;
		
		/* Go over DOM, pulling out what we need */
		for e_section in root.children() {
			match e_section.name().as_ref() {
				"title" => {
					let title_text = format!("{0} - {1}", e_section.text(), filename);
					title = Some(title_text.to_string());
				},
				
				"trackList" => {
					for e_track in e_section.children() {
						if let Ok(track) = Track::from_xml_elem(e_track) {
							tracklist.push(track);
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
	
	/* Utility - Number of digits required for padding track numbers
	 * so all filenames will have the same length for the track-number
	 * prefix.
	 */
	pub fn track_index_width(&self) -> usize
	{
		match self.len() {
			0   ..= 99   => 2,
			100 ..= 999  => 3, /* just in case */
			_            => 4  /* it's unlikely we need more */
		}
	}
	
	/* Utility - Total duration of all tracks
	 * NOTE: This returns both the duration that can be tallied, 
	 *       along with a count of how many couldn't be counted
	 */
	pub fn total_duration(&self) -> XspfDurationTallyResult
	{
		let mut result = XspfDurationTallyResult { duration: TrackDuration(0), uncounted: 0 };
		
		for track in self.tracks.iter() {
			match track.duration {
				Some(ref x) => {
					let TrackDuration(d) = *x; /* ugh! It was easier destructing than trying to figure out the lifetimes shit */
					result.duration += d;
				},
				None => {
					result.uncounted += 1;
				}
			}
		}
		
		result
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
	let playlist = XspfPlaylist::from_xml_tree(root, filename);
	Some(playlist)
}

/* ********************************************** */


