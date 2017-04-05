use std::path::Path;
use std::fs;

// Given a name and version, collect the archive to the dir, where it can then
// be unarchived.
pub fn collect_package( name : String, version : String ) -> bool {
    // Hardcoded for now
    let target_dir : &Path = Path::new("./root/tmp/mutagen/tmp_dl");

    // Change this to... not be a thing
    let search_dir : &Path = Path::new("./mutagen_archive");

    if target_dir.exists() && target_dir.is_dir() {
        let pkg_name : String = format!("{}-{}.tar.xz", name, version);

        let to = target_dir.join(Path::new(&pkg_name));
        let from = search_dir.join(Path::new(&pkg_name));

        println!("Copy {:?} to {:?}", from, to);
        fs::copy(from, to);
    }

    return true;
}
