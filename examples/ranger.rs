use pkcore::util::terminal::Terminal;

fn main() {
    env_logger::init();
    // loop {
    //     read_input();
    // }
    read_input();
}

fn read_input() {
    match Terminal::receive_range("range> ") {
        Ok(_) => {
            println!("boop!");
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
