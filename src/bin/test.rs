use atto::{
    parse,
    exec,
};

fn main() {
    let prog = parse::code(r#"
        def +'' |x| |y|
            __add x y

        def print'' |@| |msg|
            __print @ msg

        def input' |@|
            __input @

        def #'' |x| |y| y

        def main |@|
            let x 100
            let add_five' |x| + x 5
            let |@ msg| input @
            print @ msg
    "#).unwrap();

    println!("{:?}", exec::ast::run(&prog));
}
