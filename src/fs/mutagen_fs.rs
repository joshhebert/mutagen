extern crate fuse;
extern crate libc;
extern crate walkdir;
extern crate time;
use self::time::Timespec;
use std::fs::metadata;
use std::collections::HashMap;
use self::walkdir::WalkDir;
use self::walkdir::DirEntry;
use std::path::Path;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::collections::hash_map::Entry::Vacant;
use std::collections::hash_map::Entry::Occupied;
use std::path::PathBuf;
use self::libc::ENOENT;
use std::mem;
use self::fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry,
ReplyDirectory};

#[derive(Debug)]
pub enum MutagenFilesystemError {
    FileDoesNotExist,
    DirDoesNotExist,
}

pub enum Type {
    Dir,
    File,
}

#[derive (Clone)]
pub struct Tag {
    pub owner_name      : String,
    pub owner_version   : String,
}


struct Entry {
    ino        : u64,
    entry_type : Type,
}

struct DirNode {
    entries : HashMap<OsString, Entry>,
}

struct FileNode {
    true_path : PathBuf,
    tag       : Tag,
}

pub struct MutagenFilesystem {
    // Map inodes to dirs
    dir_vfs : HashMap<u64, DirNode>,

    // Map inodes to files
    file_vfs : HashMap<u64, FileNode>,

    // We maintain a mapping of paths to inodes, so we can group things
    // that are in the same directory to the same virtual inode
    mapping : HashMap<PathBuf, u64>,
    ino_counter : u64,
}

impl MutagenFilesystem {
    pub fn new() -> MutagenFilesystem {
        let mut m = MutagenFilesystem {
            dir_vfs : HashMap::new(),
            file_vfs : HashMap::new(),
            mapping : HashMap::new(),
            ino_counter : 2,
        };

        // Create a root entry
        let e = DirNode{
            entries : HashMap::new(),
        };

        m.dir_vfs.insert(1, e);
        let mut root = PathBuf::new();
        root.push("/");
        m.mapping.insert(PathBuf::new(), 1);
        m.mapping.insert(root, 1);

        return m;
    }


    fn map_inode( &mut self, parent_dir : PathBuf, entry_type : Type, name : OsString ) -> u64 {
        // Figure out what the ino of the parent dir is
        println!("Mapping {:?} into {:?}", name, parent_dir );
        let parent_ino : u64;
        match self.mapping.entry(parent_dir) {
            Occupied(mut o) => parent_ino = o.get_mut().clone(),
            Vacant(v) => panic!("Unsupported"),
        }

        // Get the DirEntry represented by this ino
        // Insert a new record into the DirEntry. If it already exists,
        // there's a conflict
        match self.dir_vfs.entry(parent_ino) {
            Occupied(mut d) => {
                let parent = d.get_mut();
                match parent.entries.entry(name){
                    Occupied(o) => panic!("File conflict"),
                    Vacant(v) => {
                        let e = Entry{
                            ino : self.ino_counter,
                            entry_type : Type::Dir,
                        };
                        v.insert(e);
                    }
                }
            }
            Vacant(v) => panic!("Unsupported"),
        }

        let ret = self.ino_counter;
        self.ino_counter += 1;

        return ret;
    }

    fn load_dir(&mut self, name : OsString, parent_dir : PathBuf, entry : DirEntry ) {
        let ino = self.map_inode( parent_dir.clone(), Type::Dir, name.clone() );

        let full_path = &parent_dir.join(Path::new(&name));
        println!("Inserting {:?} into mapping", full_path);
        self.mapping.insert( full_path.clone(), ino );

        match self.dir_vfs.entry(ino) {
            Occupied(mut o) => (),
            Vacant(v) => {
                let mut n : DirNode = DirNode {
                    entries : HashMap::new(),
                };
                v.insert(n);
            }
        }
    }


    fn load_file(&mut self, name : OsString, parent_dir : PathBuf, true_path : PathBuf, tag : Tag, entry : DirEntry ) {
        let ino = self.map_inode( parent_dir, Type::Dir, name );

        match self.file_vfs.entry(ino){
            Occupied(o) => panic!("File conflict"),
            Vacant(v) => {
                let e = FileNode{
                    true_path : true_path,
                    tag : tag,
                };
                v.insert(e);
            }
        }
    }


    /**
     * Provided with a path (presumably containing a package), index it into
     * this filesystem under the provided tag
     */
    pub fn inject(&mut self, p : &Path, tag : Tag){
        // Walk the new path
        for entry in WalkDir::new( p ) {

            let entry = entry.unwrap();
            let entry_clone = entry.clone();
            let entry_path = entry_clone.path();
            let true_path;
            if entry_path.is_absolute(){
                true_path = entry_path;
            }else{
                // We need to resolve the current path
                // getcwd + canonicalize
                panic!("Cannot use relative paths (for now)");
            }

            let name : OsString = true_path.file_name().unwrap().to_owned();
            let e : Entry = Entry{
                ino : self.ino_counter,
                entry_type : Type::Dir,
            };
            self.ino_counter += 1;

            // Get the path relative to the root of this branch
            let mut parent_dir : PathBuf = PathBuf::new();

            let mut depth = entry.depth();
            let mut iter = true_path.components().rev();

            if depth > 0 {
                // Discard first element to not include the actual name
                // of the file
                iter.next();
                for c in iter {
                    if depth <= 1 {
                        break;
                    }
                    depth -= 1;

                    // Push the next entry to the begining of the local
                    // path
                    let pc = parent_dir.clone();
                    let p = pc.as_path();
                    parent_dir = PathBuf::new();
                    parent_dir.push(Path::new(&c));
                    parent_dir.push(p);
                }

                if true_path.is_dir(){
                    self.load_dir( name, parent_dir, entry );
                }else if true_path.is_file(){
                    self.load_file( name, parent_dir, true_path.to_path_buf(), tag.clone(), entry );
                }
            }


        }
    }

    pub fn remove(&mut self, tag : Tag){

    }

    pub fn is_dir( &self, ino : u64 ) -> bool {
        return match self.dir_vfs.get(&ino) {
            Some( node ) => true,
            None => false,
        };
    }

    pub fn is_file( &self, ino : u64 ) -> bool {
        return match self.file_vfs.get(&ino) {
            Some( node ) => true,
            None => false,
        };
    }

    pub fn resolve_file_by_ino( &self, ino : u64 ) -> Result<PathBuf, MutagenFilesystemError> {
        let true_path : Result<PathBuf, MutagenFilesystemError> = match self.file_vfs.get(&ino) {
            Some( node ) => {
                let mut b = PathBuf::new();
                b.push(&(node.true_path));
                Ok(b)
            },
            None => Err(MutagenFilesystemError::FileDoesNotExist),
        };
        return true_path;
    }
    pub fn read_dir_by_ino( &self, ino : u64 ) -> Result<Vec<(OsString, u64)>, MutagenFilesystemError> {
        let contents : Result<Vec<(OsString,u64)>, MutagenFilesystemError> = match self.dir_vfs.get(&ino) {
            Some( node ) => {
                let mut v= vec!();
                for name in node.entries.keys() {
                    v.push((name.clone(), node.entries.get(name).unwrap().ino));
                }
                Ok(v)
            },
            None => Err(MutagenFilesystemError::FileDoesNotExist),
        };
        return contents;
    }
}

impl Filesystem for MutagenFilesystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
        if self.is_dir(ino){
            // In the case of a dir, we "make up" some data, as it's
            // a virtual... thing
            let mut attr: FileAttr = unsafe { mem::zeroed() };
            attr.ino = ino;
            attr.kind = FileType::Directory;
            attr.perm = 0o777;
            let ttl = Timespec::new(1, 0);
            reply.attr(&ttl, &attr);
        }

        // match self.attrs.get(&ino) {
        //     Some(attr) => {
        //         let ttl = Timespec::new(1, 0);
        //         reply.attr(&ttl, attr);
        //     }
        //     None => reply.error(ENOENT),
        // };
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup(parent={}, name={})", parent, name.to_str().unwrap());
        reply.error(ENOENT);

        // let inode = match self.inodes.get(name.to_str().unwrap()) {
        //     Some(inode) => inode,
        //     None => {
        //         reply.error(ENOENT);
        //         return;
        //     }
        // };
        // match self.attrs.get(inode) {
        //     Some(attr) => {
        //         let ttl = Timespec::new(1, 0);
        //         reply.entry(&ttl, attr, 0);
        //     }
        //     None => reply.error(ENOENT),
        // };
    }

    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, size: u32, reply: ReplyData) {
        println!("read(ino={}, fh={}, offset={}, size={})", ino, fh, offset, size);

        // for (key, &inode) in &self.inodes {
        //     if inode == ino {
        //         let value = &self.tree[key];
        //         reply.data(value.pretty().to_string().as_bytes());
        //         return;
        //     }
        // }
        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, mut reply: ReplyDirectory) {
        println!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);

        let inodes = self.read_dir_by_ino( ino ).unwrap();
        let mut offsetctr = 2;
        for i in inodes {
            if self.is_dir( i.1 ){
                reply.add( i.1, offsetctr, FileType::Directory, i.0 ); 
            }else if self.is_file( i.1 ){
                reply.add( i.1, offsetctr, FileType::RegularFile, i.0 ); 
            }
        }

        reply.add(1, 0, FileType::Directory, ".");
        reply.add(1, 1, FileType::Directory, "..");

        reply.ok();
    }
}

#[cfg(target_family="unix")]
pub fn mount_fs() {
    let mut fs = MutagenFilesystem::new();

    fs.inject(Path::new("/home/spooky/dev/mutagen/pkg"), Tag{owner_name: "test".to_string(), owner_version: "1.0".to_string()});
    // fs.inject(Path::new("/home/josh/devel/mutagen/old_work"), Tag{owner_name: "test2".to_string(), owner_version: "1.0".to_string()});

    // println!("{:?}", fs.resolve_file_by_ino(12));
    // println!("{:?}", fs.lookup(Path::new("python_fuse_system/fuse_logic.py")));
    // println!("{:?}", fs.lookup(Path::new("python_fuse_system")));

    let mountpoint = "./mount";

    fuse::mount(fs, &mountpoint, &[]).expect("Couldn't mount filesystem");
}

