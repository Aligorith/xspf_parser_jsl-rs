/* Parser for XSPF files
 *
 * This is simply a wrapper around an underlying XML reading library,
 * so that we can just abstract out the bits we want to expose.
 */
//#[macro_use] extern crate lazy_static;
extern crate regex;
use self::regex::Regex;

extern crate minidom;
use self::minidom::Element;

use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::path::Path;
use std::fmt;

/* ********************************************** */
/* Utility Types */

/* Track Duration */
pub struct TrackDuration(i64);

impl TrackDuration {
	/* Convert from milliseconds to seconds */
	fn to_secs(&self) -> f64
	{
		let TrackDuration(ms) = *self;
		(ms as f64) / 1000.0_f64
	}
	
	/* Convert from milliseconds to minutes */
	fn to_mins(&self) -> f64
	{
		let secs = self.to_secs();
		secs / 60.0_f64
	}
	
	/* Convert from milliseconds to "mins:secs" timecode string */
	fn to_timecode(&self) -> String
	{
		/* Total seconds - We don't care about the leftover milliseconds */
		let total_secs = self.to_secs() as i64;
		
		/* mins:secs */
		let mins: i64 = total_secs / 60;
		let secs: i64 = total_secs % 60;
		
		/* output string */
		format!("{0:02}:{1:02}", mins, secs)
	}
}

impl fmt::Display for TrackDuration {
	/* Display timecodes instead of raw ints when printing */
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}", self.to_timecode())
	}
}

/* XXX: how to deduplicate? */
impl fmt::Debug for TrackDuration {
	/* Display timecodes instead of raw ints when printing */
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}", self.to_timecode())
	}
}


/* ------------------------------------------- */

/* Track Types */
#[derive(Debug)]
enum TrackType {
	Unknown,
	ViolinLayering,
	MuseScore,
	Piano,
	Voice,
}

/* ------------------------------------------- */

/* Filename Extension */
#[allow(non_camel_case_types)]
enum TrackExtension {
	/* Placeholder - Only used when constructing the type */
	Placeholder,
	
	mp3,
	flac,
	ogg,
	m4a,
	mp4,
}

/* From https://www.reddit.com/r/rust/comments/2vqama/parse_string_as_enum_value/cojzafn/
 * Usage: string.parse::<TrackExtension>()
 */
impl FromStr for TrackExtension {
	type Err = (&'static str);
	
    fn from_str(s: &str) -> Result<TrackExtension, Self::Err> {
        match s {
            "mp3"  => Ok(TrackExtension::mp3),
            "flac" => Ok(TrackExtension::flac),
            "ogg"  => Ok(TrackExtension::ogg),
            "m4a"  => Ok(TrackExtension::m4a),
            "mp4"  => Ok(TrackExtension::mp4),
            _      => Err("Unknown extension")
        }
    }
}

/* ------------------------------------------- */


/* Filename Info Components
 * Internal use only, for easier extraction of interesting aspects
 */
struct FilenameInfoComponents {
	/* Track Type */
	track_type : TrackType,
	/* Sequence Index in that day's sessions */
	index : i32,
	
	/* Descriptive name (all underscores/symbols get normalised out) */
	name: String,
	
	/* filename extension */
	extn : TrackExtension
}

impl FilenameInfoComponents {
	/* Internal-Use Constructor - Run regexes on a name string (minus the extension)
	 * and generate a stub instance with the affected fields filled out
	 */
	fn from_file_stem(filename: &str) -> Self
	{
		/* Defines for the regex expressions to use - all get initialised on first run, then can be accessed readily later */
		lazy_static! {
			/* Violin Layering */
			static ref RE_ViolinLayering : Regex = Regex::new(r"^v(?P<index>\d+)-(?P<id>.+)$").unwrap();
			
			/* Muse Score */
			static ref RE_MuseScore : Regex = Regex::new(r"^(?P<date>\d{8})-(?P<index>\d+)-(?P<id>.+)$").unwrap();
		}
		
		/* Define placeholder values */
		let mut track_type = TrackType::Unknown;
		let mut index = 1;
		let mut name = filename.to_string(); //String::new();
		
		/* Try each of the regex'es to find a match */
		if RE_ViolinLayering.is_match(filename) {
			track_type = TrackType::ViolinLayering;
			// TODO: extract values out of captures
		}
		else if RE_MuseScore.is_match(filename) {
			track_type = TrackType::MuseScore;
			// TODO: extract values out of captures
		}
		
		
		/* Return new instance */
		FilenameInfoComponents {
			track_type : track_type,
			index : index,
			name : name.to_string(),
			extn : TrackExtension::Placeholder,
		}
	}
	
	
	/* Constructor from filename */
	fn new(filename: &str) -> Self
	{
		/* Use Path to split the "name" portion from the extension */
		let path = Path::new(filename);
		let name_part: &str = path.file_stem().unwrap()  /* OsString - This should be ok to unwrap like this */
		                          .to_str().unwrap();    /* &str - Need to unwrap the converted version to get what we need */
		
		/* Generate the stub instance, with all the name-parts filled out */
		let mut fic = Self::from_file_stem(name_part);
		
		/* Extract the extension info */
		let extn_str = path.extension().unwrap()    /* get OsString */
		                   .to_str().unwrap();      /* get &str - Need to unwrap the converted version */
		let extn = extn_str.parse::<TrackExtension>()
		                   .unwrap();               /* get contents of mandatory Result */
		
		/* ... and set extension now */
		fic.extn = extn;
		
		/* Return new instance */
		fic
	}
}

/* ********************************************** */
/* Playlist Types */

/* A track listing in the playlist */
#[derive(Debug)]
pub struct Track {
	/* Descriptive name assigned to this track */
	pub name: String,
	/* Duration (in ms) of the track - as stored in the file */
	pub duration: Option<TrackDuration>,
	
	/* Full name of the track itself (v<num>_<name>.<mp3/flac>) */
	pub filename: String,
	/* Date string of the track (i.e. parent directory) */
	pub date: String,
	
	/* Full path (extracted from the file) */
	pub path: String,
}

const FILE_URI_PREFIX: &'static str = "file:///";

impl Track {
	/* Generate a track element from a file path */
	pub fn from_filepath(path: &str) -> Result<Track, &'static str>
	{
		/* full unmodfied path */
		let fullpath = path.to_string();
		
		/* extra filename and date from the last parts of the path 
		 * WARNING: We're extracting these in reverse order! So first filename, then date!
		 */
		// TODO: Sanity checking!
		let mut path_elems : Vec<&str> = fullpath.split("/").collect();
		
		let filename = path_elems.pop().unwrap().to_string();
		let date = path_elems.pop().unwrap().to_string();
		
		let name_info = FilenameInfoComponents::new(filename.as_ref());
		
		
		/* Construct and return a track */
		Ok(Track {
			name: name_info.name,
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


