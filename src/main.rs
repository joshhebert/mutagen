#![feature(dedup_by)]

use std::collections::HashMap;
use std::ascii::AsciiExt;
mod version;
use version::Version;

mod package_resolver;
use package_resolver::Resolver;
use package_resolver::FilesystemResolver;


struct Context<T>{
    map : HashMap<String, Node>,
    resolver : T,
}

impl<T : Resolver> Context<T>{
    fn new(rs : T) -> Context<T> {
        let hm : HashMap<String, Node> = HashMap::new();
        let mut e = Context { resolver : rs, map : hm };
        e.add_node("ROOT");
        return e;
    }

    fn flatten( &self, start : String ) -> Vec<(String, Version)> {
        // Start at the node identified by start and collect its
        let mut ret : Vec<(String, Version)> = vec!();
        let mut deps : Vec<String> = vec!();
        match self.map.get( &start ) {
            Some(n) => {
                let mut node_deps_iter = n.deps.iter();
                loop {
                    match node_deps_iter.next() {
                        Some(d) => {
                            deps.push(d.to_owned());
                        },
                        None => break,
                    }
                }
            },
            None => {}
        };

        // Resolve the things at the top level
        if deps.len() > 0 {
            let mut i = deps.iter();
            loop {
                match i.next() {
                    Some(s) => {
                        let v = self.get_target_version( s.to_owned() );
                        ret.push( (s.to_owned(),v) );
                        // For each of these, recurse and append
                        let subdeps = self.flatten( s.to_owned() );
                        ret = [ret, subdeps].concat();
                    },
                    None => break,
                }
            }
        }

        // Remove duplicates in the return vector
        ret.sort_by(|a,b| a.0.cmp(&(b.0)));
        ret.dedup_by(|a,b| a.0.eq_ignore_ascii_case(&(b.0)));

        return ret;
    }

    fn get_target_version( &self, name : String ) -> Version {
        // Convert the name to a node
        match self.map.get( &name ){
            Some( n ) => {
                return n.collapse_rules().max_version;
            },
            None => {
                panic!("Node could not be resolved");
            },
        }
    }

    fn inject( &mut self, name : String, version : String ){
        // Check if package exists

        // Add an explicit rule requiring THIS version
        // of the package
        let rule = Rule{ owner: "ROOT".to_string(), min_version: Version::new(&version), max_version: Version::new(&version) };
        self.add_constraint( name, rule );
    }

    fn add_constraint(&mut self, name : String, new_rule : Rule){
        // Test if this package exists in the map already
        println!("Injecting {}", name );
        // I KNOW this can be simplified TODO
        let start_v : Version;
        let node_found : bool;
        match self.map.get( &name ) {
            Some(n) => {
                start_v = n.collapse_rules().max_version;
                node_found = true;
            },
            None => {
                // If not, inject this node
                start_v = new_rule.max_version.to_owned();
                node_found = false;
            }
        };

        // If the node does not exist, create it
        if !node_found {
            self.add_node( &name );
        }

        // Insert the new rule
        self.add_rule( &new_rule.owner, &name, &new_rule.min_version.data, &new_rule.max_version.data );

        // If the target version of the package changed, we need to
        // refresh all of our rules for this node
        let end_v : Version;
        match self.map.get( &name ) {
            Some(n) => {
                end_v = n.collapse_rules().max_version;
            },
            None => {
                panic!("My node disappeared...");
            }
        };

        if start_v.cmp(&end_v) != 0 || !node_found{
            self.refresh_node( &name );
        }

    }

    ///
    /// Clean out old deps and resbuild them from the metadata
    ///
    fn refresh_node( &mut self, name : &str ){
        println!("Refreshing {}", name );
        // Figure out this node's version
        let target_v : String;
        match self.map.get( name ) {
            Some(n) => {
                target_v = n.collapse_rules().max_version.data.to_owned();
            },
            None => {
                panic!("Request on non-existant node requested");
            }
        };

        match self.resolver.resolve( name, &target_v ){
            Ok(meta) => {
                // Extract list of deps
                let deps : Vec<String>;
                match self.map.get( name ) {
                    Some(n) => {
                        deps = n.deps.to_owned();
                    },
                    None => {
                        panic!("What");
                    }
                };

                // Clear old rules
                let mut old_deps_iter = deps.iter();
                loop {
                    match old_deps_iter.next() {
                        Some(r) => {
                            self.remove_rule( name, &r );
                        },
                        None => break,
                    }
                }


                // Using the deps from the resolver, add rules originating
                // from this now
                let mut new_deps_iter = meta.deps.iter();
                loop {
                    match new_deps_iter.next() {
                        Some(r) => {
                            println!("Working on dep {}", r.name );
                            // Force the target node to re-evaluate its life
                            let new_rule = Rule{ owner : name.to_string(), min_version : Version::new(&(r.min_version)), max_version : Version::new(&(r.max_version)) };
                            self.add_constraint( r.name.to_string(), new_rule );
                        },
                        None => break,
                    }
                }

            },
            Err(_) => panic!("Could not resolve package")
        }
    }

    // Private helper functions
    ///////////////////////////////

    // TODO use entry API here
    fn add_rule<'a>( &mut self, from : &'a str, to : &'a str, min : &'a str, max : &'a str ) {
        // Ensure that both from and to exist
        if !self.map.contains_key(from) || !self.map.contains_key(to){
            panic!("Attempted to add a bad rule");
        }

        // Create new rule
        let new_rule : Rule = Rule{ min_version : Version::new(min),
                                    max_version : Version::new(max),
                                    owner       : from.to_string().clone()
                                  };

        // The panic!'s should never occur, but if they do, we should
        // abort, as it means there could be an inconsistency between
        // the from node and the to node
        match self.map.get_mut(to) {
            Some(n) => n.rules.push(new_rule),
            None => panic!("Bad thing")
        };

        match self.map.get_mut(from) {
            Some(n) => n.deps.push(to.to_string().clone()),
            None => panic!("Bad thing")
        };

    }

    ///
    /// Remove all rules owned by owner from target, and remove target as
    /// a dependency of owner
    ///
    fn remove_rule<'a>( &mut self, owner : &'a str, target : &'a str ){
        // Ensure that both from and to exist
        if !self.map.contains_key(owner) || !self.map.contains_key(target){
            panic!("Tried to remove a rule that doesn't exist");
        }

        // The panic!'s should never occur, but if they do, we should
        // abort, as it means there could be an inconsistency between
        // the from node and the to node
        match self.map.get_mut(owner) {
            Some(n) => n.deps.retain(|i| *i != target),
            None => panic!("Bad thing")
        };

        match self.map.get_mut(target) {
            Some(n) => n.rules.retain(|i| i.owner != owner),
            None => panic!("Bad thing")
        };
    }

    ///
    /// add_node(HashMap<&str,Node>, &str) -> void
    /// Given a name node_name, insert a new node into the hashmap
    ///
    fn add_node<'a>( &mut self, node_name : &'a str ) {
        let n = Node{ name : node_name.to_string().clone(), rules : vec!(), deps : vec!() };
        self.map.insert(node_name.to_string().clone(), n);

    }
}


struct Rule{
    min_version : Version,
    max_version : Version,
    owner       : String,
}

struct Node {
    name : String,
    rules : Vec<Rule>,
    deps : Vec<String>,
}
impl Node{
    /// Iterate over all the max versions of the rules to find
    /// the lowest
    fn collapse_rules(&self) -> Rule {
        // We better have rule to collapse
        assert!(self.rules.len() > 0);

        let mut lowest_max : &Version = &self.rules[0].max_version;
        let mut iter = self.rules.iter();
        loop {
            match iter.next() {
                Some(r) => {
                    if r.max_version.cmp(&lowest_max) == -1 {
                        lowest_max = &r.max_version;
                    }
                },
                None => break,
            }
        }

        let mut highest_min : &Version = &self.rules[0].min_version;
        iter = self.rules.iter();
        loop {
            match iter.next() {
                Some(r) => {
                    if r.min_version.cmp(&highest_min) == 1 {
                        highest_min = &r.min_version;
                    }
                },
                None => break,
            }
        }

        assert!( highest_min.cmp( lowest_max ) == -1 ||
                 highest_min.cmp( lowest_max ) == 0
               );


        let max = Version{ data : lowest_max.data.clone() };
        let min = Version{ data : highest_min.data.clone() };

        return Rule{ max_version: max,
                     min_version: min,
                     owner: "nobody".to_string()
                   };
    }
}

fn main() {
    let mut c = Context::new(FilesystemResolver{});
    c.inject("emacs".to_string(), "3.0".to_string());
    c.inject("vim".to_string(), "4.5".to_string());



    println!("Testing");
    match c.map.get_mut("ROOT") {
        Some(n) => println!("{}", n.deps.len()),
        None => return
    }
    let d = c.flatten("ROOT".to_string());


    println!("Resolved tree:");
    let mut i = d.iter();
    loop {
        match i.next() {
            Some(r) => {
                println!("{}", r.0);
            },
            None => break,
        }
    }
}
