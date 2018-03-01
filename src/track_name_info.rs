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
#[derive(PartialEq)]
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
#[derive(PartialEq)]
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
 *
 * Provides a mechanism for extracting of interesting aspects
 * contained within track filenames
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
/* Unit Tests */

#[cfg(test)]
mod tests {
	use super::*;
	
	/* Support Type Tests --------------------------------------------------------------- */
	
	/* Check that the TrackExtension string->enum parsing works correctly */
	#[test]
	fn test_filename_extensions()
	{
		assert_eq!(TrackExtension::mp3,   "mp3".parse::<TrackExtension>().unwrap());
		assert_eq!(TrackExtension::flac,  "flac".parse::<TrackExtension>().unwrap());
		assert_eq!(TrackExtension::ogg,   "ogg".parse::<TrackExtension>().unwrap());
		assert_eq!(TrackExtension::m4a,   "m4a".parse::<TrackExtension>().unwrap());
		assert_eq!(TrackExtension::mp4,   "mp4".parse::<TrackExtension>().unwrap());
	}
	
	/* Check that the TrackType shortname stuff works as expected */
	#[test]
	fn test_tracktype_shortname()
	{
		assert_eq!("?",   TrackType::UnknownType.shortname());
		assert_eq!("t",   TrackType::UnknownType.shortname_safe());
		
		assert_eq!("VL",  TrackType::ViolinLayering.shortname());
		assert_eq!("VL",  TrackType::ViolinLayering.shortname_safe());
		
		assert_eq!("MS",  TrackType::MuseScore.shortname());
		assert_eq!("MS",  TrackType::MuseScore.shortname_safe());
		
		assert_eq!("P",   TrackType::Piano.shortname());
		assert_eq!("P",   TrackType::Piano.shortname_safe());
		
		assert_eq!("V",   TrackType::Voice.shortname());
		assert_eq!("V",   TrackType::Voice.shortname_safe());
	}
	
	/* Check that violin-layering filenames parse correctly ----------------------------- */
	
	/* Check that simple violin-layering filenames parse correctly */
	#[test]
	fn test_violin_basic()
	{
		let v1 = FilenameInfoComponents::new("v01-tranquil.mp3");
		assert_eq!(TrackType::ViolinLayering, v1.track_type);
		assert_eq!(1, v1.index);
		assert_eq!("tranquil", v1.name);
		assert_eq!(TrackExtension::mp3, v1.extn);
		
		let v2 = FilenameInfoComponents::new("v02-celestial.mp3");
		assert_eq!(TrackType::ViolinLayering, v2.track_type);
		assert_eq!(2, v2.index);
		assert_eq!("celestial", v2.name);
		assert_eq!(TrackExtension::mp3, v2.extn);
		
		let v3 = FilenameInfoComponents::new("v03-spectral.mp3");
		assert_eq!(TrackType::ViolinLayering, v3.track_type);
		assert_eq!(3, v3.index);
		assert_eq!("spectral", v3.name);
		assert_eq!(TrackExtension::mp3, v3.extn);
	}
	
	/* Check that multiword violin layering filenames parse correctly */
	#[test]
	fn test_violin_multiword()
	{
		let v1 = FilenameInfoComponents::new("v02-winds_of_flutter.mp3");
		assert_eq!(TrackType::ViolinLayering, v1.track_type);
		assert_eq!(2, v1.index);
		assert_eq!("winds_of_flutter", v1.name);
		assert_eq!(TrackExtension::mp3, v1.extn);
	}
	
	/* Check that multiversion violin layering filenames parse correctly */
	#[test]
	fn test_violin_multiversion()
	{
		let v1 = FilenameInfoComponents::new("v01a-outcrop.mp3");
		assert_eq!(TrackType::ViolinLayering, v1.track_type);
		assert_eq!(1, v1.index);
		// XXX: Variant numbers are not currently extracted and stored
		assert_eq!("outcrop", v1.name);
		assert_eq!(TrackExtension::mp3, v1.extn);
		
		let v2 = FilenameInfoComponents::new("v05L-wild_west.mp3");
		assert_eq!(TrackType::ViolinLayering, v2.track_type);
		assert_eq!(5, v2.index);
		// XXX: Variant numbers are not currently extracted and stored
		assert_eq!("wild_west", v2.name);
		assert_eq!(TrackExtension::mp3, v2.extn);
	}
	
	/* Older-Style Violin-Layering names (circa 2016)*/
	#[test]
	fn test_vln_improv()
	{
		//"vln_improv_04-mystique.mp3"
		//"vln_improv_01.mp3"
	}
	
	#[test]
	fn test_vln_layering()
	{
		//"vln_layering-05-the_last_moose.mp3"
		//"vln_layering-03-delicate.mp3"
	}
	
	/* Check that musescore filenames parse correctly ----------------------------------- */
	
	#[test]
	fn test_ms_basic()
	{
		
	}
	
	#[test]
	fn test_ms_multiword()
	{
		let m1 = FilenameInfoComponents::new("20170802-02-TouchedByAnAngel.flac");
		assert_eq!(TrackType::MuseScore, m1.track_type);
		assert_eq!(2, m1.index);
		assert_eq!("TouchedByAnAngel", m1.name);
		assert_eq!(TrackExtension::flac, m1.extn);
		
		let m2 = FilenameInfoComponents::new("20170815-05-CanadianBeauty.flac");
		assert_eq!(TrackType::MuseScore, m2.track_type);
		assert_eq!(5, m2.index);
		assert_eq!("CanadianBeauty", m2.name);
		assert_eq!(TrackExtension::flac, m2.extn);
	}
	
	#[test]
	fn test_ms_multiversion_postfix()
	{
		/* Names where the version is included in a postfix after the name */
		//"20170821-03-MajesticSerenade-v2.flac"
		//"20170801-01-Patterns-WIP"
	}
}

/* *************************************************** */

