mod lib;

use clap::{App, Arg};
use git2;
use std::path::Path;

fn main() {
    let matches = App::new("add2git")
        .version("0.1.1")
        .author("SAGUYWALKER <saguywalker@protonmail.com>")
        .about("CLI application to fetch, pull, add, commit and push a file to GIT without running the command sequentially.")
	.arg(
	    Arg::with_name("FILE")
		.required(true)
		.multiple(true)
		.index(1)
		.help("The file(s) you would like to add"),
	)
        .arg(
            Arg::with_name("credentialpath")
                .short("c")
                .long("credentialpath")
                .takes_value(true)
                .required(false)
                .help("A path to your ssh key (default: ~/.ssh/id_rsa)"),
        )
        .arg(
            Arg::with_name("commit")
                .short("m")
                .long("commit")
                .takes_value(true)
                .required(false)
                .help("A commit message (default: add $FILE)"),
        )
        .arg(
            Arg::with_name("branch")
                .short("b")
                .long("branch")
                .takes_value(true)
                .required(false)
                .help("A branch to commit (default: master)"),
        )
        .get_matches();

    //handling filename
    let filenames: Vec<String> = matches.values_of("FILE").unwrap().map(|x| lib::validate_file(Some(x)).unwrap()).collect();
   
    //handling credential file
    let priv_file = lib::validate_credfile(matches.value_of("credentialpath")).unwrap();
    //println!("Private key file: {}.", priv_file.display());

    //handling public key file
    let mut priv_filename = String::from(priv_file.to_str().unwrap());
    priv_filename.push_str(".pub");
    let pub_file = if Path::new(&priv_filename.as_str()).exists() {
        Some(Path::new(priv_filename.as_str()))
    } else {
        None
    };
    //println!("Public key file: {:?}", pub_file);

    //handling commit message
    let commit_msg = match matches.value_of("commit") {
        Some(msg) => String::from(msg),
        None => String::from("add ") + &filenames.join(" "),
    };
    //println!("Commit message: {}", commit_msg);

    //handling branch
    let branch = matches.value_of("branch").unwrap_or("master");

    //open a repository
    let repo = git2::Repository::open(".").expect("Could not open a repository.");
    //println!("{} stat={:?}", repo.path().display(), repo.state());

    let mut remote = repo
        .find_remote("origin")
        .expect("Could not find origin remote");

    //fetch repository
    let fetch_commit = lib::fetch_repository(&repo, &mut remote, &pub_file, &priv_file)
        .expect("Could not fetch a repository.");
    //println!("Fetch complete");
    //merge
    lib::do_merge(&repo, &branch, fetch_commit).expect("Could not merge");
    //println!("Merge complete");

    //add new file and commit
    let _commit_id = lib::add_and_commit(&repo, filenames, commit_msg.as_str())
        .expect("Couldn't add file to repo");
    //println!("New commit: {}", _commit_id);

    //push file
    lib::push(&mut remote, &branch, &pub_file, &priv_file).expect("Could not push");
    println!("Push file(s) successfully\n");

    //display recently commit
    let commit = lib::find_last_commit(&repo).expect("Could not find the last commit");
    lib::display_commit(&commit);
 
}
