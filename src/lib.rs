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
) -> Result<git2::Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    //let signature = git2::Signature::now(user, email)?;
    let signature = repo.signature()?;
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

pub fn fetch_repository<'a>(repo: &'a git2::Repository, remote: &'a mut git2::Remote, pub_file: &Option<&Path>, priv_file: &Path) -> Result<git2::AnnotatedCommit<'a>, git2::Error>{
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut fetch_options = git2::FetchOptions::new();
    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key("git", *pub_file, &priv_file, None)
            .expect("Could not create credentials object");
        Ok(credentials)
    });
    fetch_options.remote_callbacks(callbacks);

    remote.fetch(&["master"], Some(&mut fetch_options), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

pub fn merge_branch(repo: &git2::Repository, local: &git2::AnnotatedCommit, remote: &git2::AnnotatedCommit) -> Result<(), git2::Error>{
    let local_tree = repo.find_tree(local.id())?;
    let remote_tree = repo.find_tree(remote.id())?;
    let ancestor = repo.find_tree(repo.merge_base(local.id(), remote.id())?)?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;
    if idx.has_conflicts() {
        println!("Merge conficts detected...");
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

pub fn push<'a>(repo: &git2::Repository, pub_file: &Option<&Path>, priv_file: &Path) -> Result<(), git2::Error>{
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key("git", *pub_file, &priv_file, None)?;
        Ok(credentials)
    });
    let mut push_ops = git2::PushOptions::new();
    push_ops.remote_callbacks(callbacks);
    let mut remote = repo
        .find_remote("origin")?;
    remote.push(&["refs/heads/master:refs/heads/master"],Some(&mut push_ops))?;
    Ok(())
}
