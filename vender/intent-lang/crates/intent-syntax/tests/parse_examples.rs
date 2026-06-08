use intent_syntax::parse;

fn parse_file(path: &str) {
    let src = std::fs::read_to_string(path).unwrap();
    match parse(&src) {
        Ok(prog) => {
            eprintln!("  ✅ {} — {} declarations", path, prog.declarations.len());
        }
        Err(e) => {
            panic!("  ❌ {}: {}", path, e);
        }
    }
}

#[test]
fn parse_transfer_example() {
    parse_file("../../examples/basics/transfer.intent");
}

#[test]
fn parse_auth_example() {
    parse_file("../../examples/basics/auth.intent");
}

#[test]
fn parse_sorting_example() {
    parse_file("../../examples/basics/sorting.intent");
}

#[test]
fn parse_smarthome_example() {
    parse_file("../../examples/smarthome/smarthome.intent");
}
