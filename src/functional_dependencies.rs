use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Attribute(String);

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
impl From<BTreeSet<Attribute>> for AttributeSet {
    fn from(x: BTreeSet<Attribute>) -> Self {
        Self(x)
    }
}
impl Into<BTreeSet<Attribute>> for AttributeSet {
    fn into(self) -> BTreeSet<Attribute> {
        self.0
    }
}
impl ::std::str::FromStr for AttributeSet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split(",").map(str::trim).map(Attribute::from).collect(),
        ))
    }
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Dependency {
    left: AttributeSet,
    right: AttributeSet,
}

impl Dependency {
    pub fn from_set_pair((l, r): (AttributeSet, AttributeSet)) -> Self {
        Self { left: l, right: r }
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

    pub fn minimize(&self, deps: &DependencySet) -> Dependency {
        let mut min = self.clone();

        for attr in min.left().set().clone().into_iter() {
            min.left_mut().remove(&attr);

            if !min.left().closure(deps).is_superset(self.right()) {
                min.left_mut().insert(attr);
            }
        }

        min
    }
}

impl Into<(AttributeSet, AttributeSet)> for Dependency {
    fn into(self) -> (AttributeSet, AttributeSet) {
        (self.left, self.right)
    }
}

impl ::std::fmt::Display for Dependency {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{} -> {}", self.left, self.right)
    }
}

impl ::std::str::FromStr for Dependency {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split("->");

        Ok(Self {
            left: parts.next().ok_or(())?.parse()?,
            right: parts.next().ok_or(())?.parse()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DependencySet(HashSet<Dependency>);

impl DependencySet {
    pub fn effective_attributes(&self) -> AttributeSet {
        self.clone()
            .0
            .into_iter()
            .map(|d| d.into())
            .map(|(l, r)| (l.into(), r.into()))
            .map(|(l, r): (BTreeSet<_>, BTreeSet<_>)| l.into_iter().chain(r.into_iter()))
            .flatten()
            .collect::<BTreeSet<_>>()
            .into()
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
            cover = Self(
                union_map
                    .into_iter()
                    .map(Dependency::from_set_pair)
                    .collect(),
            );

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
                let min = dep.minimize(&cover);
                if min != dep {
                    dep = min;
                    changed = true;
                }

                cover.insert(dep);
            }

            if !changed {
                break;
            }
        }

        cover
    }

    pub fn candidate_keys(&self, attributes: &AttributeSet) -> Vec<AttributeSet> {
        let mut keys = Vec::new();

        keys.push(
            (Dependency::from_set_pair((attributes.clone(), attributes.clone()))
                .minimize(self)
                .into(): (_, _))
                .0,
        );

        let mut n = 1;
        let mut i = 0;

        while i < n {
            for dep in self.iter() {
                let new_key =
                    AttributeSet::from(dep.left().set() | &(keys[i].set() - dep.right().set()));

                if !keys.iter().any(|k| k.is_subset(&new_key)) {
                    keys.push(new_key);

                    n += 1;
                }
            }

            i += 1;
        }

        keys
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

impl ::std::str::FromStr for DependencySet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut deps = HashSet::new();

        for line in s.trim().lines() {
            let dep: Dependency = line.parse()?;
            deps.insert(dep);
        }

        Ok(Self(deps))
    }
}
