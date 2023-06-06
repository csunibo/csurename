use csurename::Config;
use std::process;

fn main() {
    println!("csurename:  Running in filter mode, empty line \'$^\' or CTRL-C to quit.\n");
    //But how to correctly impl / wrap Config?
    //
    //New pub constructors as big as this one cannot be the answer.
    //Help me

    let config = Config::new_filter().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = csurename::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
