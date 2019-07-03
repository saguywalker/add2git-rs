use clap::{App, Arg};
use git2;
use std::{fs::File, io::Write, path::Path, process, env};
use time;

fn main() {
    let matches = App::new("add2git-rs")
        .version("0.1.0")
        .author("SAGUYWALKER <guyguy252@gmail.com>")
        .about("CLI programming to add, commit and push the file(s) to Git")
        .arg(Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .takes_value(true)
                 .required(false)
                 .help("The file(s) you would like to add"))
        .arg(Arg::with_name("cred")
                 .short("c")
                 .long("cred")
                 .takes_value(true)
                 .required(false)
                 .help("A path to your ssh key"))
        .arg(Arg::with_name("repo")
                 .short("r")
                 .long("repo")
                 .takes_value(true)
                 .required(false)
                 .help("A repository url"))
        .get_matches();

    let filename = validate_file(matches.value_of("file")).unwrap();
    println!("File {} is found.", filename.display());

    let priv_file = validate_file(matches.value_of("cred")).unwrap();
    println!("{} is found.", priv_file.display());
    let mut priv_filename = String::from(priv_file.to_str().unwrap());
    priv_filename.push_str(".pub");
    let pub_file = Path::new(priv_filename.as_str());

    //let repo_url = matches.value_of("repo").expect("please enter the repository url");
    let repo_clone_path = "workspace/";
    //println!("Cloning {} into {}", repo_url, repo_clone_path);

    //let mut builder = git2::build::RepoBuilder::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut fetch_options = git2::FetchOptions::new();
    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key(
            "git",
            Some(&pub_file),
            &priv_file,
            None,
        )
        .expect("Could not create credentials object");
        Ok(credentials)
    });

    fetch_options.remote_callbacks(callbacks);
    
    //builder.fetch_options(fetch_options);
    /*let repo = builder
        .clone(repo_url, Path::new(repo_clone_path))
        .expect("Could not clone a repo");
    println!("Clone complete");*/
    let repo = git2::Repository::discover(Path::new(repo_clone_path)).expect("workspace is not discovered");

    let mut remote = repo.find_remote("origin").expect("Error with finding remote");
    remote.fetch(&["master"], Some(&mut fetch_options), None).expect("Could not fetch");

    let commit = find_last_commit(&repo).expect("Could not find the last commit");
    display_commit(&commit);
    /*
    {
        let file_path = std::env::current_dir()
            .unwrap()
            .join(repo_clone_path)
            .join(filename);
        let mut file = File::create(file_path.clone()).expect("Couldn't create file");
        file.write_all(b"Testing with git2").unwrap();
    }*/

    let mut commit_msg = String::from("add ");
    commit_msg.push_str(filename.to_str().unwrap());

    let mut repo_path = env::current_dir().unwrap();
    repo_path.push("workspace");
    let strip_filename = &filename.strip_prefix(&repo_path).expect("Could not stip the file");

    let commit_id = add_and_commit(&repo, &strip_filename, commit_msg.as_str())
        .expect("Couldn't add file to repo");
    println!("New commit: {}", commit_id);

    let mut callbacks2 = git2::RemoteCallbacks::new();
    callbacks2.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key(
            "git",
            Some(&pub_file),
            &priv_file,
            None,
        )
        .expect("Could not create credentials object");
        Ok(credentials)
    });

    let mut push_ops = git2::PushOptions::new();
    push_ops.remote_callbacks(callbacks2);

    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_ops),
        )
        .expect("error with pushing files");
    
}

fn validate_file<'a>(filename: Option<&'a str>) -> Result<&'a Path, &'static str>{
    match filename{
        None => Err("please enter filename."),
        Some(f) => {
            if Path::new(f).exists(){
                return Ok(Path::new(f))
            }else{
                return Err("input file does not exist.")
            };
        }
    }
}

fn find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn display_commit(commit: &git2::Commit) {
    let timestamp = commit.time().seconds();
    let tm = time::at(time::Timespec::new(timestamp, 0));
    println!(
        "commit {}\nAuthor: {}\nDate:  {}\n\n   {}",
        commit.id(),
        commit.author(),
        tm.rfc822(),
        commit.message().unwrap_or("no commit message...")
    );
}

fn add_and_commit(
    repo: &git2::Repository,
    path: &Path,
    message: &str,
) -> Result<git2::Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = git2::Signature::now("saguywalker", "guyguy252@gmail.com")?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )
}

fn push(repo: &git2::Repository, url: &str) -> Result<(), git2::Error> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", url)?,
    };
    remote.connect(git2::Direction::Push)?;
    remote.push(&["refs/heads/master:refs/heads/master"], None)
}
