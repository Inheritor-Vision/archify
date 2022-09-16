use clap::{Parser, ErrorKind, CommandFactory};

#[derive(Parser)]
#[clap(author="Inheritor-Vision")]
#[clap(version)]
#[clap(about = "Tool that will periodically save user personnal playlist and public playlists. It is aimed at keeping record of temporary playlist like Weekly Discovery, made by Spotify, or public playlist.", long_about = None)]
struct Cli {
	/// Launch user register process
	#[clap(long, value_parser)]
	add_user: Option<String>,
	/// Add public playlist to archive
	#[clap(long, value_parser)]
	add_playlist: Option<String>,
	/// Update playlist stored in database
	#[clap(short,long,action,value_parser)]
	update: bool,
	/// Delete a playlist
	#[clap(short,long,value_parser)]
	delete_playlist: Option <String>,
}

pub enum Args {
	NewUser(String),
	NewPlaylist(String),
	DeletePlaylist(String),
	Update,
	NewUserId,
}

pub fn parse_args() -> Args{
	let cli = Cli::parse();
	let res;

	if cli.update && cli.add_user == None && cli.add_playlist == None && cli.delete_playlist == None{
		res = Args::Update;
	}else if !cli.update && cli.add_user != None && cli.add_playlist == None && cli.delete_playlist == None {
		res = Args::NewUser(cli.add_user.unwrap());
	}else if !cli.update && cli.add_user == None && cli.add_playlist != None && cli.delete_playlist == None {
		res = Args::NewPlaylist(cli.add_playlist.unwrap());
	}else if !cli.update && cli.add_user == None && cli.add_playlist == None && cli.delete_playlist != None {
		res = Args::DeletePlaylist(cli.delete_playlist.unwrap());
	}else{
		let mut cmd = Cli::command();
		cmd.error(
			ErrorKind::ArgumentConflict,
			"Choose only one the available option!"
		).exit()
	}
	res
}