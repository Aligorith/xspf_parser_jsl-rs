xspf_parser_jsl-rs
==================
*Rust-based tool for processing XSPF playlists, extracting metadata embedded in the filenames, and exporting this data into various formats that are easier to use*


Usage
-----

`` $ xspf_tools {mode} {in.xspf} [{outfile/dir}] ``
                  
where {mode} is one of the following:
   * **help**    -  Prints this text
   
   * **dump**    -   Prints summary of the important identifying info gained from the playlist
   * **runtime** -   Prints summary of the total running time of the playlist
   
   * **list**    -   Writes the file paths of all tracks in the playlist to {outfile}
   * **json**    -   Extracts the useful info out of the file, and dumps to JSON format
                      in {outfile} for easier handling
   
   * **copy**    -  Copies all the files named in the playlist to the nominated folder {outdir}
                     Their names will get prefixed with metadata such as the track number and date.


What is this - Motivation/Purpose
---------------------------------
Over the past year, I recorded/composed over 3 hours of music for fun and as a way to de-stress during a hectic few months of writing up and defending my thesis.
These tracks (each ~ 1 minute long) were saved into a bunch of XSPF playlists (one per month), in a carefully curated order, to facilitate easy playback of these
sequences in VLC. Thanks to these tracks/playlists, I remained enough sanity to plough through the thesis writing (over 260 pages of it) and make it out alive.

As one of my post-graduation projects, I figured it would be fun to try and share this collection with the world in the form of some automatically-generated music
videos. But since we're talking about several hundred tracks here, it's not really that fun manually hunting down and loading all these files into a video sequencer,
then manually extracting the useful information out of the filenames to populate various slots in the titlecards used to introduce/identify each track. Hence, the need
for some automation!

Normally, I'd have written such a project in Python (and in some ways, it would've been easier). However, I soon realised that this was a great opportunity to try writing
some real code in Rust. I'd also been interested in working on some "real" projects in Rust for a while - as simple toy programs really don't go far enough when it comes to
trying to figure out the ins and outs of a language - so it seemed like a great idea to accomplish both goals b using both on this project!


-- Joshua Leung (@Aligorith), March 2018



