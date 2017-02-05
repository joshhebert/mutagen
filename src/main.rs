use std::collections::HashMap;

mod version;
use version::Version;

mod package_resolver;
use package_resolver::Resolver;
use package_resolver::FilesystemResolver;
use package_resolver::DummyResolver;


struct Ecosystem<T>{
    map : HashMap<String, Node>,
    resolver : T,
}

impl<T : Resolver> Ecosystem<T>{
    fn new(rs : T) -> Ecosystem<T> {
        let mut hm : HashMap<String, Node> = HashMap::new();
        let mut e = Ecosystem { resolver : rs, map : hm };
        e.add_node("ROOT");
        return e;
    }

    fn pin( &mut self, name : String, version : String ){
        // Check if package exists

        // Add an explicit rule requiring THIS version
        // of the package
        let rule = Rule{ owner: "ROOT".to_string(), min_version: Version::new(&version), max_version: Version::new(&version) };
        self.inject( name, rule );
    }

    fn inject(&mut self, name : String, new_rule : Rule){
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
        // Stupid borrow checker
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
                            self.inject( r.name.to_string(), new_rule );
                        },
                        None => break,
                    }
                }

            },
            Err(_) => panic!("Could not resolve package")
        }
    }

    ///
    /// This function bothers me a lot, as it doesn't elegantly check
    /// for key existance and *then* update, so we end up basically
    /// testing twice. Furthermore, I'd prefer that owner of Rule be a
    /// direct reference to the node, rather than a string that has to be
    /// looked up every time.
    ///
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

    // e.add_node( "ROOT" );
    // e.add_node( "vim" );
    // e.add_rule( "ROOT", "vim", "1.0", "2.0" );
    // e.add_rule( "ROOT", "vim", "1.4", "2.5" );
    // e.add_rule( "ROOT", "vim", "0.5", "1.6" );

    // match e.map.get_mut("vim") {
    //     Some(n) => {
    //         let r = n.collapse_rules();
    //         println!( "{}, {}", r.min_version.data, r.max_version.data );
    //     },
    //     None => {}
    // };
    // remove_rule( &mut map, "ROOT", "vim" );



    // let map : HashMap<String, Node> = HashMap::new();
    // let r = FilesystemResolver{};
    // let e = Ecosystem{ map : map, resolver : r };
    let mut e = Ecosystem::new(FilesystemResolver{});
    e.pin("emacs".to_string(), "3.0".to_string());
    e.pin("vim".to_string(), "4.5".to_string());
    println!("Testing");
    match e.map.get_mut("ROOT") {
        Some(n) => println!("{}", n.deps.len()),
        None => return
    }
    // let v = Version::new("1.0.2.k-1".to_string());
    // let v2 = Version::new("1.0.2.j-1".to_string());
    // assert!( v.cmp(&v2) == 1 );
}
