use pkcore::util::terminal::Terminal;

fn main() {
    println!("Hello");
    Terminal::pause("Press any key to continue...\r\n");
    println!("World!");
}
