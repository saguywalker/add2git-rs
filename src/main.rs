use git2;
use std::{fs::File, io::Write, path::Path};
use time;

fn main() {
    let repo_url = "git@github.com:saguywalker/go-with-drone.git";
    let repo_clone_path = "workspace/";
    println!("Cloning {} into {}", repo_url, repo_clone_path);

    let mut builder = git2::build::RepoBuilder::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut fetch_options = git2::FetchOptions::new();

    callbacks.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key(
            "git",
            Some(Path::new("/path/to/id_rsa.pub")),
            Path::new("/path/to/id_rsa"),
            None,
        )
        .expect("Could not create credentials object");
        Ok(credentials)
    });

    fetch_options.remote_callbacks(callbacks);
    builder.fetch_options(fetch_options);

    let repo = builder
        .clone(repo_url, Path::new(repo_clone_path))
        .expect("Could not clone a repo");
    println!("Clone complete");

    let commit = find_last_commit(&repo).expect("Couldn't find last commit");
    display_commit(&commit);

    let relative_path = Path::new("example.txt");
    {
        let file_path = std::env::current_dir()
            .unwrap()
            .join(repo_clone_path)
            .join(relative_path);
        let mut file = File::create(file_path.clone()).expect("Couldn't create file");
        file.write_all(b"Testing with git2").unwrap();
    }
    let commit_id = add_and_commit(&repo, &relative_path, "add example.txt")
        .expect("Couldn't add file to repo");
    println!("New commit: {}", commit_id);

    let mut callbacks2 = git2::RemoteCallbacks::new();
    callbacks2.credentials(|_, _, _| {
        let credentials = git2::Cred::ssh_key(
            "git",
            Some(Path::new("/path/to/id_rsa.pub")),
            Path::new("/path/to/id_rsa"),
            None,
        )
        .expect("Could not create credentials object");
        Ok(credentials)
    });

    let mut push_ops = git2::PushOptions::new();
    push_ops.remote_callbacks(callbacks2);

    let mut remote = repo.find_remote("origin").expect("error with finding remote");
    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_ops),
        )
        .expect("error with pushing files");
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
