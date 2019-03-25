use atto::parse;

fn main() {
    let prog = parse::code(r#"
        def makes_2.
            if true
                (4 5 8)
            9
    "#).unwrap();

    println!("{:?}", prog);
}
