/* Types and Utilities for decomposing track filenames
 * to extract out all the metadata they contain
 */
//#[macro_use] extern crate lazy_static;
extern crate regex;
use self::regex::Regex;

use std::fmt;
use std::str::FromStr;
use std::path::Path;

/* *************************************************** */

/* Track Types */
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum TrackType {
	UnknownType,
	ViolinLayering,
	MuseScore,
	Piano,
	Voice,
}

impl TrackType {
	/* Get an abbreviated name for more compact display */
	pub fn shortname(&self) -> String
	{
		match *self {
			TrackType::UnknownType    => "?".to_string(),
			TrackType::ViolinLayering => "VL".to_string(),
			TrackType::MuseScore      => "MS".to_string(),
			TrackType::Piano          => "P".to_string(),
			TrackType::Voice          => "V".to_string(),
		}
	}
	
	/* Get abbreviated name that's safe for use in filenames */
	pub fn shortname_safe(&self) -> String
	{
		match *self {
			/* Only the "unknown" type needs special handling right now... */
			TrackType::UnknownType   => "t".to_string(),
			
			/* Everything else can use the standard way */
			_                        => self.shortname(),
		}
	}
}

/* *************************************************** */

/* Filename Extension */
#[derive(Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum TrackExtension {
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

/* *************************************************** */

/* Filename Info Components
 * Internal use only, for easier extraction of interesting aspects
 */
#[derive(Serialize, Deserialize)]
pub struct FilenameInfoComponents {
	/* Track Type */
	pub track_type : TrackType,
	/* Sequence Index in that day's sessions */
	pub index : i32,
	
	/* Descriptive name (all underscores/symbols get normalised out) */
	pub name: String,
	
	/* filename extension */
	pub extn : TrackExtension
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
			static ref RE_VIOLIN_LAYERING : Regex = Regex::new(r"^v(?P<index>\d+)(?P<variant>[[:alpha:]]?)-(?P<id>.+)$").unwrap();
			
			/* Muse Score */
			static ref RE_MUSE_SCORE : Regex      = Regex::new(r"^(?P<date>\d{8})(?P<variant>[[:alpha:]]?)-(?P<index>\d+)-(?P<id>.+)$").unwrap();
		}
		
		/* Try each of the regex'es to find a match */
		if let Some(vcap) = RE_VIOLIN_LAYERING.captures(filename) {
			/* return Violin Layering case */
			let index = vcap["index"].parse::<i32>()
									 .unwrap_or_default();
			let name  = vcap["id"].to_string(); // XXX: Prettify
			
			FilenameInfoComponents {
				track_type : TrackType::ViolinLayering,
				index : index,
				name : name,
				extn : TrackExtension::Placeholder,
			}
		}
		else if let Some(mcap) = RE_MUSE_SCORE.captures(filename) {
			/* return MuseScore case */
			let index = mcap["index"].parse::<i32>()
									 .unwrap_or_default();
			let name  = mcap["id"].to_string(); // XX: Prettify
			
			FilenameInfoComponents {
				track_type : TrackType::MuseScore,
				index : index,
				name : name,
				extn : TrackExtension::Placeholder,
			}
		}
		else {
			let track_type = TrackType::UnknownType;
			let index = 0;
			let name = filename.to_string(); //String::new();
			
			/* Return new instance */
			FilenameInfoComponents {
				track_type : track_type,
				index : index,
				name : name.to_string(),
				extn : TrackExtension::Placeholder,
			}
		}
	}
	
	
	/* Constructor from filename */
	pub fn new(filename: &str) -> Self
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

impl fmt::Debug for FilenameInfoComponents {
	/* Display key info from FilenameInfoComponents */
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, r"[{0}]  idx={1}, n='{2}', ext={3:?}",
			   self.track_type.shortname(),
			   self.index,
			   self.name,
			   self.extn)
	}
}

/* *************************************************** */

