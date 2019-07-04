use git2;
use std::{env, path::Path, path::PathBuf, process::Command};
use time;

pub fn validate_file<'a>(filename: Option<&'a str>) -> Result<String, &'static str> {
    match filename {
        None => Err("Please enter filename."),
        Some(f) => {
            let abs_path = env::current_dir().unwrap().join(Path::new(f));
            if abs_path.exists() {
                return Ok(String::from(f));
            } else {
                return Err("Input file does not exist.");
            };
        }
    }
}

pub fn validate_credfile<'a>(filename: Option<&'a str>) -> Result<PathBuf, &'static str> {
    match filename {
        None => {
            let home = match env::var("HOME") {
                Ok(val) => {
                    let cred_path = Path::new(val.as_str()).join(".ssh").join("id_rsa");
                    if !cred_path.exists() {
                        return Err("Please enter a path to your credential file.");
                    }
                    Ok(cred_path.to_path_buf())
                }
                Err(_) => Err("Please enter a path to your credential file."),
            };
            home
        }
        Some(f) => {
            let cred_path = Path::new(f);
            if cred_path.exists() {
                return Ok(cred_path.to_path_buf());
            } else {
                return Err("Credential file does not exist.");
            };
        }
    }
}

pub fn get_default_signature(mode: &str) -> Result<String, &'static str> {
    let git_command = match mode {
        "email" => "git config --get user.email",
        "name" => "git config --get user.name",
        _ => panic!("Error with signature mode"),
    };
    let vec_user_signature = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", git_command])
            .output()
            .expect("failed to execute process")
            .stdout
    } else {
        Command::new("sh")
            .args(&["-c", git_command])
            .output()
            .expect("failed to execute process")
            .stdout
    };
    if vec_user_signature.len() == 0 {
        return Err("Failed to read the git config, please provide it directly");
    }
    Ok(String::from(
        std::str::from_utf8(&vec_user_signature[..vec_user_signature.len() - 1]).unwrap(),
    ))
}

pub fn find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

pub fn display_commit(commit: &git2::Commit) {
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

pub fn add_and_commit(
    repo: &git2::Repository,
    path: &Path,
    message: &str,
    user: &str,
    email: &str,
) -> Result<git2::Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = git2::Signature::now(user, email)?;
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

pub fn fetch_repository<'a>(repo: &git2::Repository, pub_file: &Option<&Path>, priv_file: &Path) {
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut fetch_options = git2::FetchOptions::new();
    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key("git", *pub_file, &priv_file, None)
            .expect("Could not create credentials object");
        Ok(credentials)
    });
    fetch_options.remote_callbacks(callbacks);

    let mut remote = repo
        .find_remote("origin")
        .expect("Error with finding remote");
    remote
        .fetch(&["master"], Some(&mut fetch_options), None)
        .expect("Could not fetch");
    println!("Fetch repository successfully.")
}

pub fn push<'a>(repo: &git2::Repository, pub_file: &Option<&Path>, priv_file: &Path) {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key("git", *pub_file, &priv_file, None)
            .expect("Could not create credentials object");
        Ok(credentials)
    });
    let mut push_ops = git2::PushOptions::new();
    push_ops.remote_callbacks(callbacks);
    let mut remote = repo
        .find_remote("origin")
        .expect("Error with finding remote");
    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_ops),
        )
        .expect("error with pushing files");
    println!("Push file successfully.")
}
