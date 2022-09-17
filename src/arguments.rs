use clap::{Parser, ErrorKind, CommandFactory};

#[derive(Parser)]
#[clap(author="Inheritor-Vision")]
#[clap(version)]
#[clap(about = "Tool that will periodically save user personnal playlist and public playlists. It is aimed at keeping record of temporary playlist like Weekly Discovery, made by Spotify, or public playlist.", long_about = None)]
struct Cli {
	/// Launch user register authorization process. Need Auth code & used redirect url
	#[clap(long, value_parser, min_values = 2, max_values = 2)]
	add_user: Option<Vec<String>>,
	/// Add public playlist to archive
	#[clap(long, value_parser, min_values = 1, max_values = 2)]
	add_playlist: Option<Vec<String>>,
	/// Get ID of the app for spotify API
	#[clap(long,action,value_parser)]
	get_client_id: bool,
	/// Update playlist stored in database
	#[clap(short,long,action,value_parser)]
	update: bool,
	/// Delete a playlist
	#[clap(short,long,value_parser)]
	delete_playlist: Option <String>,
}

pub enum Args {
	NewUser(String, String),
	NewPlaylist(Vec<String>),
	DeletePlaylist(String),
	Update,
	GetClientId,
}

pub fn parse_args() -> Args{
	let cli = Cli::parse();
	let res;

	if cli.update {
		res = Args::Update;
	}else if cli.add_user != None {
		let v = cli.add_user.unwrap();
		res = Args::NewUser(v[0].clone(), v[1].clone());
	}else if cli.add_playlist != None {
		res = Args::NewPlaylist(cli.add_playlist.unwrap());
	}else if cli.delete_playlist != None {
		res = Args::DeletePlaylist(cli.delete_playlist.unwrap());
	}else if cli.get_client_id {
		res = Args::GetClientId;
	}else{
		let mut cmd = Cli::command();
		cmd.error(
			ErrorKind::ArgumentConflict,
			"Choose only one the available option!"
		).exit()
	}
	res
}