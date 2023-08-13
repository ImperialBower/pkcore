use std::io;
use std::io::Write;

#[must_use]
pub fn receive_usize(prompt: &str) -> usize {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to receive value");
    let trimmed = input_text.trim();
    match trimmed.parse::<usize>() {
        Ok(i) => i,
        Err(..) => 0,
    }
}