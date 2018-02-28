/* Implementation of "Track Duration" type */
use std::fmt;
use std::ops::{Add, AddAssign};

/* Track Duration */
#[derive(Serialize, Deserialize)]
pub struct TrackDuration(pub i64);

#[allow(dead_code)]
impl TrackDuration {
	/* Convert from milliseconds to seconds */
	pub fn to_secs(&self) -> f64
	{
		let TrackDuration(ms) = *self;
		(ms as f64) / 1000.0_f64
	}
	
	/* Convert from milliseconds to minutes */
	pub fn to_mins(&self) -> f64
	{
		let secs = self.to_secs();
		secs / 60.0_f64
	}
	
	/* Convert from milliseconds to "mins:secs" timecode string */
	pub fn to_timecode(&self) -> String
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


/* Operator Overrides - The standard cases */
impl Add for TrackDuration {
	type Output = TrackDuration;
	fn add(self, other: TrackDuration) -> TrackDuration
	{
		let TrackDuration(x) = self;
		let TrackDuration(y) = other;
		TrackDuration(x + y)
	}
}
impl AddAssign for TrackDuration {
	fn add_assign(&mut self, other: TrackDuration)
	{
		let TrackDuration(x) = *self;
		let TrackDuration(y) = other;
		
		*self = TrackDuration(x + y);
	}
}


/* Operator Overrides - The useful cases */
impl Add<i64> for TrackDuration {
	type Output = TrackDuration;
	fn add(self, other: i64) -> TrackDuration
	{
		let TrackDuration(old_val) = self;
		TrackDuration(old_val + other)
	}
}
impl AddAssign<i64> for TrackDuration {
	fn add_assign(&mut self, other: i64)
	{
		let TrackDuration(old_val) = *self;
		*self = TrackDuration(old_val + other);
	}
}


/* Display Formatting - We want it to display as a timecode (instead of a plain number in milliseconds) */
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

