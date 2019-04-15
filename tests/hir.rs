use atto::{
    parse,
    exec,
};

fn main() {
    let code = r#"
        def +'' x -> y ->
            __add x y

        def add'' a -> b ->
            + a b

        def print'' @ -> msg ->
            __print @ msg

        def main @ ->
            add 10 15
    "#;

    let hir = parse::code(code)
        .unwrap()
        .to_hir();

    println!("HIR (unoptimised)\n{:?}\n", hir);

    println!("HIR (optimised once)\n{:?}\n", (0..4).fold(hir, |hir, _| hir.optimise_once()));
}
