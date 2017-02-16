use solver::version::Version;

pub struct Node {
    pub name : String,
    pub rules : Vec<Rule>,
    pub deps : Vec<String>,
}

pub struct Rule{
    pub min_version : Version,
    pub max_version : Version,
    pub owner       : String,
}

impl Node{
    /// Iterate over all the max versions of the rules to find
    /// the lowest
    pub fn collapse_rules(&self) -> Rule {
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
