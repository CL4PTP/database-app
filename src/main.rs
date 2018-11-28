extern crate dbapp;

use dbapp::functional_dependencies::*;

fn main() {
    let fd: DependencySet = "
        A,B -> C,D
        C->E,F
        A->F
        E->F
    "
    .parse()
    .unwrap();

    println!("fd: {}", fd);
    println!("attributes: {}", fd.effective_attributes());

    let attr: AttributeSet = "A".parse().unwrap();

    println!("attr: {}", attr);
    println!("closure: {}", attr.closure(&fd));

    println!("minimal cover: {}", fd.minimal_cover());

    println!("key closures:");
    for key in fd.candidate_keys(&fd.effective_attributes()) {
        println!("{} -> {}", key, key.closure(&fd));
    }
}
