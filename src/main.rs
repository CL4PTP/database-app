#![feature(self_struct_ctor)]
#![feature(nll)]

extern crate indexmap;

mod functional_dependencies {
    use std::collections::{BTreeSet, HashMap, HashSet};

    #[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Attribute(String);

    impl Attribute {
        pub fn from_simple_form(c: char) -> Self {
            Self(c.to_string())
        }
    }

    impl<'a> From<&'a str> for Attribute {
        fn from(v: &str) -> Self {
            Self(v.to_string())
        }
    }

    impl ::std::fmt::Display for Attribute {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl ::std::ops::Deref for Attribute {
        type Target = String;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl ::std::ops::DerefMut for Attribute {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
    pub struct AttributeSet(BTreeSet<Attribute>);

    impl AttributeSet {
        pub fn from_simple_form(s: &str) -> Self {
            Self(s.chars().map(Attribute::from_simple_form).collect())
        }

        pub fn set(&self) -> &BTreeSet<Attribute> {
            &self.0
        }

        pub fn set_mut(&mut self) -> &mut BTreeSet<Attribute> {
            &mut self.0
        }

        /*
        decomposition rule:     if X->YZ then X->Y and X->Z
        transitive rule:        if X->Y and Y->Z then X->Z
        union rule:             if X->Y and X->Z, then X->YZ
        augmentation rule:      if X->Y then WX->WY
        pseudo-transitive rule: if X->Y and WY->Z then WX->Z
        */
        pub fn closure(&self, deps: &DependencySet) -> AttributeSet {
            // apply trivial reflexive rule
            let mut closure = self.clone();

            'outer: loop {
                let mut changed = false;
                // transitive rule (by implied decomposition rule)
                for d in deps.iter() {
                    if closure.is_superset(&d.left()) && !closure.is_superset(&d.right()) {
                        closure.0 = closure.union(&d.right()).cloned().collect();
                        changed = true;
                    }
                }

                // union rule (by implied decomposition rule)
                for d in deps.iter() {
                    if self.is_superset(&d.left()) && !closure.is_superset(&d.right()) {
                        closure.0 = closure
                            .union(&d.right())
                            .cloned()
                            .collect::<BTreeSet<Attribute>>();
                        changed = true;
                    }
                }

                if !changed {
                    break;
                }
            }

            closure
        }
    }

    impl ::std::fmt::Display for AttributeSet {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(
                f,
                "{}",
                self.0
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }

    impl ::std::ops::Deref for AttributeSet {
        type Target = BTreeSet<Attribute>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl ::std::ops::DerefMut for AttributeSet {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Dependency {
        left: AttributeSet,
        right: AttributeSet,
    }

    impl Dependency {
        pub fn from_set_pair((l, r): (AttributeSet, AttributeSet)) -> Self {
            Self {
                left: l,
                right: r,
            }
        }

        pub fn from_simple_form(s: (&str, &str)) -> Self {
            Self {
                left: AttributeSet::from_simple_form(s.0),
                right: AttributeSet::from_simple_form(s.1),
            }
        }

        pub fn left(&self) -> &AttributeSet {
            &self.left
        }

        pub fn left_mut(&mut self) -> &mut AttributeSet {
            &mut self.left
        }

        pub fn right(&self) -> &AttributeSet {
            &self.right
        }

        pub fn right_mut(&mut self) -> &mut AttributeSet {
            &mut self.right
        }
    }

    impl Into<(AttributeSet, AttributeSet)> for Dependency {
        fn into(self) -> (AttributeSet, AttributeSet) { (self.left, self.right) }
    }

    impl ::std::fmt::Display for Dependency {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "{} -> {}", self.left, self.right)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct DependencySet(HashSet<Dependency>);

    impl DependencySet {
        pub fn from_simple_form(deps: &[(&str, &str)]) -> Self {
            Self(
                deps.iter()
                    .map(|s| Dependency::from_simple_form(*s))
                    .collect(),
            )
        }

        // as seen in `CIS611_LectureNotes_8_MinimalCover.pdf`
        pub fn minimal_cover(&self) -> DependencySet {
            // remove trivial dependencies
            let mut cover = Self(
                self.iter()
                    .cloned()
                    .filter(|dep| dep.left() != dep.right())
                    .collect(),
            );

            'outer: loop {
                // apply union rule
                let mut union_map = HashMap::new();
                for dep in cover.0.into_iter() {
                    let (l, r) = dep.into();

                    let v = union_map.entry(l).or_insert_with(|| r.clone());
                    **v = v.union(&r).cloned().collect::<BTreeSet<Attribute>>();
                }
                cover = Self(union_map.into_iter().map(Dependency::from_set_pair).collect());

                let mut changed = false;

                for mut dep in cover.0.clone().into_iter() {
                    cover.remove(&dep);

                    // remove extraneous RHS attributes
                    for attr in dep.right().set().clone().into_iter() {
                        dep.right_mut().set_mut().remove(&attr);

                        cover.insert(dep.clone());

                        let contains = dep.left().closure(&cover).contains(&attr);

                        cover.remove(&dep);

                        if !contains {
                            dep.right_mut().set_mut().insert(attr);
                        } else {
                            changed = true;
                        }
                    }

                    // remove extraneous LHS attributes
                    for attr in dep.left().set().clone().into_iter() {
                        dep.right_mut().set_mut().remove(&attr);

                        if !dep.left().closure(&cover).is_superset(dep.right()) {
                            dep.left_mut().set_mut().insert(attr);
                        } else {
                            changed = true;
                        }
                    }

                    cover.insert(dep);
                }

                if !changed {
                    break;
                }
            }

            cover
        }

        // need to get it below O(2^n) worst case
        pub fn candidate_keys(&self) -> Vec<Dependency> {
            unimplemented!()
        }
    }

    impl ::std::fmt::Display for DependencySet {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "F {{")?;
            writeln!(
                f,
                "{}",
                self.iter()
                    .map(ToString::to_string)
                    .map(|s| "\n\t".to_string() + &s)
                    .collect::<Vec<String>>()
                    .concat()
            )?;
            write!(f, "}}")
        }
    }

    impl ::std::ops::Deref for DependencySet {
        type Target = HashSet<Dependency>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl ::std::ops::DerefMut for DependencySet {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

fn main() {
    let fd = functional_dependencies::DependencySet::from_simple_form(&[
        ("A", "BC"),
        ("B", "CE"),
        ("A", "E"),
        ("AC", "H"),
        ("D", "B"),
    ]);

    println!("fd: {}", fd);

    let attr = functional_dependencies::AttributeSet::from_simple_form("A");

    println!("attr: {}", attr);
    println!("closure: {}", attr.closure(&fd));

    println!("minimal cover: {}", fd.minimal_cover());
}
