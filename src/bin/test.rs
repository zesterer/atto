use atto::parse;

fn main() {
    let prog = parse::code(r#"
        def ['' |items| |t|
            __cat __wrap items t

        def ,'' |x| |y|
            __cat __wrap x y

        def ]
            let foo 5
            let |bar foobar| [1, 3]
            __tail __wrap null

        def test
            [1, 2, 3, 4]

        def +'' |x| |y|
            __add x y

        def ''' |f'| |arg|
            f arg

        def add_five' |x|
            '$+ 5

        def main
            add_five 10
    "#).unwrap();

    println!("{:?}", prog);
}
