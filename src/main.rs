mod lib;

use clap::{App, Arg};
use git2;
use std::{path::Path};

fn main() {
    let matches = App::new("add2git-rs")
        .version("0.1.0")
        .author("SAGUYWALKER <guyguy252@gmail.com>")
        .about("CLI programming to add, commit and push a file to Git")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .required(true)
                .help("A file you would like to add"),
        )
        .arg(
            Arg::with_name("credentialpath")
                .short("c")
                .long("credentialpath")
                .takes_value(true)
                .required(false)
                .help("A path to your ssh key"),
        )
        .arg(
            Arg::with_name("commit")
                .short("m")
                .long("commit")
                .takes_value(true)
                .required(false)
                .help("A commit message"),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .takes_value(true)
                .required(false)
                .help("A username signature"),
        )
        .arg(
            Arg::with_name("email")
                .short("e")
                .long("email")
                .takes_value(true)
                .required(false)
                .help("An email signature"),
        )
        .get_matches();

    //handling filename
    let filename = lib::validate_file(matches.value_of("file")).unwrap();
    println!("File {} is found.", filename);

    //handling credential private key file
    let priv_file = lib::validate_credfile(matches.value_of("credentialpath")).unwrap();
    println!("Private key file: {}.", priv_file.display());

    //handling public key file
    let mut priv_filename = String::from(priv_file.to_str().unwrap());
    priv_filename.push_str(".pub");
    let pub_file = if Path::new(&priv_filename.as_str()).exists() {
        Some(Path::new(priv_filename.as_str()))
    } else {
        None
    };
    println!("Public key file: {:?}", pub_file);

    //handling commit message
    let commit_msg = match matches.value_of("commit") {
        Some(msg) => String::from(msg),
        None => {
            let mut tmp = String::from("add ");
            tmp.push_str(&filename);
            tmp.as_str();
            format!("add {}", &filename)
        }
    };
    println!("Commit message: {}", commit_msg);

    //handling user
    let username = match matches.value_of("user"){
        Some(s) => String::from(s),
        None => lib::get_default_signature("name").unwrap(),
    };
    let email = match matches.value_of("email"){
        Some(s) => String::from(s),
        None => lib::get_default_signature("email").unwrap(),
    };
    println!("{}, {}", username, email);

    let repo = git2::Repository::open(".").expect("Could not open a repository.");
    println!("{} stat={:?}", repo.path().display(), repo.state());

    //fetch repository
    lib::fetch_repository(&repo, &pub_file, &priv_file);

    //add new file and commit
    let commit_id = lib::add_and_commit(
        &repo,
        Path::new(&filename),
        commit_msg.as_str(),
        username.as_str(),
        email.as_str(),
    )
    .expect("Couldn't add file to repo");
    println!("New commit: {}", commit_id);

    //push file
    lib::push(&repo, &pub_file, &priv_file);

    //display recently commit
    let commit = lib::find_last_commit(&repo).expect("Could not find the last commit");
    lib::display_commit(&commit);
}
