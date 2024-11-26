use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::symlink_metadata;
use std::os::unix::fs::lchown;
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;

use clap::Parser;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(about = "Shift user and group IDs of files and folders")]
struct Cli {
    #[arg(long, help = "Disables recursion into sub-directories")]
    no_recursive: bool,
    #[arg(short, long, help = "Enable verbose command output)")]
    verbose: bool,
    #[arg(long, help = "Skips changing the user and group IDs for trial runs")]
    dry_run: bool,
    #[arg(short, long, allow_negative_numbers = true, value_parser = clap::value_parser!(i64).range(-i64::from(u32::MAX)..=i64::from(u32::MAX)), help = "The user ID offset to be applied to each file. Can both be positive or negative")]
    uid_offset: i64,
    #[arg(short, long, allow_negative_numbers = true, value_parser = clap::value_parser!(i64).range(-i64::from(u32::MAX)..=i64::from(u32::MAX)), help = "The group ID offset to be applied to each file. Can both be positive or negative")]
    gid_offset: i64,
    #[arg(required = true, num_args = 1.., help = "Paths which user and group IDs should get shifted")]
    paths: Vec<PathBuf>,
}

type Inode = u64;
type DevId = u64;
type PathRef = (Inode, DevId);

fn shift_owner(path: &OsStr, seen_files: &mut HashSet<PathRef>, args: &Cli) -> Result<(), String> {
    let attrs = symlink_metadata(path)
        .map_err(|err| format!("Could not load file metadata for {path:?}: {err:?}"))?;

    let path_ref: PathRef = (attrs.ino(), attrs.dev());
    if !seen_files.insert(path_ref) {
        return Ok(());
    }

    let old_user_id = attrs.uid();
    let old_group_id = attrs.gid();

    let new_user_id = u32::try_from(i64::from(old_user_id) + args.uid_offset).map_err(|err| {
        format!(
            "UID offset results in invalid new UID for {path:?}: {err:?} (old UID: {old_user_id})"
        )
    })?;

    let new_group_id = u32::try_from(i64::from(old_group_id) + args.gid_offset).map_err(|err| {
        format!(
            "GID offset results in invalid new GID for {path:?}: {err:?} (old GID: {old_group_id}"
        )
    })?;

    if args.verbose {
        println!("Changing uid:gid from {old_user_id}:{old_group_id} to {new_user_id}:{new_group_id} for {path:?}");
    }

    if !args.dry_run {
        lchown(path, Some(new_user_id), Some(new_group_id))
            .map_err(|err| format!("Could not set new owner for {path:?}: {err:?}"))?;
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();
    let mut seen_files: HashSet<PathRef> = HashSet::new();

    for path in &args.paths {
        if args.no_recursive {
            if let Err(err) = shift_owner(path.as_ref(), &mut seen_files, &args) {
                eprintln!("{err}");
            }
            continue;
        }

        for entry in WalkDir::new(path) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!("Error while looping over entry: {err}");
                    continue;
                }
            };

            if let Err(err) = shift_owner(entry.into_path().as_ref(), &mut seen_files, &args) {
                eprintln!("{err}");
            }
        }
    }
}
