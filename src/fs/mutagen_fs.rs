extern crate fuse;
extern crate libc;
extern crate walkdir;
use std::collections::HashMap;
use self::walkdir::WalkDir;
use std::path::Path;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::collections::hash_map::Entry::Vacant;
use std::collections::hash_map::Entry::Occupied;
use std::path::PathBuf;
use self::libc::ENOENT;
use self::fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry,
ReplyDirectory};

pub struct Tag {
    pub owner_name      : String,
    pub owner_version   : String,
}

struct Entry {
    true_loc: String,
    tag     : Tag,
}

struct Node {
    entries : HashMap<String, Entry>,
}

pub struct MutagenFilesystem {
    vfs : HashMap<String, Node>,
}

impl MutagenFilesystem {
    pub fn new() -> MutagenFilesystem {
        MutagenFilesystem {
            vfs : HashMap::new(),
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
            let entry_path = entry.path();
            let path;
            if entry_path.is_absolute(){
                path = entry_path;
            }else{
                // We need to resolve the current path
                // getcwd + canonicalize
                panic!("Cannot use relative paths (for now)");
            }
            // We have the full path of this file, but we'll need the
            // path relative to the root
            let mut depth = entry.depth();
            let mut iter = path.components().rev();

            if path.is_file() {
                // If this is a file, add it to our tree
                let name : OsString = path.file_name().unwrap().to_owned();
                let true_loc : &str = path.to_str().unwrap();
                let e : Entry = Entry{
                    true_loc : true_loc.to_string(),

                    // Clone is kinda nasty, make this a reference to
                    // the tag?
                    tag : Tag{
                        owner_name: tag.owner_name.to_owned(),
                        owner_version: tag.owner_version.to_owned(),
                    },
                };

                // Get the path relative to the root of this branch
                let mut local_path : PathBuf = PathBuf::new();

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
                    let pc = local_path.clone();
                    let p = pc.as_path();
                    local_path = PathBuf::new();
                    local_path.push(Path::new(&c));
                    local_path.push(p);
                }

                // If the node at local_path exists, insert the entry
                // Otherwise, create a new node with this entry in it
                println!("Testing {}", local_path.to_str().unwrap().to_string());
                match self.vfs.entry(local_path.to_str().unwrap().to_string()) {
                    Occupied(mut o) => {
                        o.get_mut().entries.insert(
                            name.to_str().unwrap().to_string(),
                            e
                        );
                    },
                    Vacant(v) => {
                        let mut n : Node = Node {
                            entries : HashMap::new(),
                        };
                        n.entries.insert(
                            name.to_str().unwrap().to_string(),
                            e
                        );
                        v.insert(n);
                    }
                }
                println!("{:?}", local_path);
            }
        }

    }

    pub fn remove(&mut self, tag : Tag){

    }

    pub fn lookup_file( &self, p : &Path ) {
        // Remove the filename itself
        let parent = p.parent().unwrap().to_str().unwrap().to_string() + "/";
        let base   = p.file_name().unwrap().to_str().unwrap().to_string();
        println!("Looking up {}", parent);

        let true_path = match self.vfs.get(&parent) {
            Some( node ) => {
                match node.entries.get( &base ) {
                    Some( e ) => println!("{}", e.true_loc),
                    None => panic!("Ack"),
                }
            },
            None => {
                panic!("Not ready");
            },
        };
    }
}

impl Filesystem for MutagenFilesystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
        reply.error(ENOENT);

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


        // if ino == 1 {
        //     if offset == 0 {
        //         reply.add(1, 0, FileType::Directory, ".");
        //         reply.add(1, 1, FileType::Directory, "..");
        //         for (key, &inode) in &self.inodes {
        //             if inode == 1 {
        //                 continue;
        //             }
        //             let offset = inode; // hack
        //             println!("\tkey={}, inode={}, offset={}", key, inode, offset);
        //             reply.add(inode, offset, FileType::RegularFile, key);
        //         }
        //     }
        //     reply.ok();
        // } else {
        //     reply.error(ENOENT);
        // }
        reply.error(ENOENT);
    }
}

#[cfg(target_family="unix")]
fn main() {
    let fs = MutagenFilesystem::new();
    let mountpoint = "./mount";

    fuse::mount(fs, &mountpoint, &[]).expect("Couldn't mount filesystem");
}

