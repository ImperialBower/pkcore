use regex::Regex;

fn main() {
    // let (act, rest) = next_act("r200ffcfc");
    //
    // println!("{act} {rest}");
    let acts = parse("r200ffcfc".to_string());
    let acts = parse_cards("r200ffcfc");
    println!("{:?}", acts);
}

pub fn parse(s: String) -> Vec<String> {
    let mut acts: Vec<String> = Vec::new();
    let re = Regex::new(r"(<acts>[cfr][0-9]*)").unwrap();
    let mut res = re.captures_iter(s.as_str());

    for cap in res {
        acts.push(cap["acts"].to_string())
    }

    acts
}

pub fn next_act(s: &str) -> (String, String) {
    let re = Regex::new(r"^(?<act>[cfr][0-9]*)(?<rest>.*)$").unwrap();
    let mut res = re.captures_iter(s);

    let caps = match res.next() {
        None => return (String::new(), String::new()),
        Some(c) => c,
    };

    (caps["act"].to_string(), caps["rest"].to_string())
}

fn parse_cards(s: &str) -> Vec<String> {
    let mut acts: Vec<String> = Vec::new();
    let mut rest = String::new();
    let (act, replace) = next_act(s);
    acts.push(act);

    loop {
        println!("{rest}");
        let (act, replace) = next_act(rest.clone().as_str());
        acts.push(act);
        rest.clear();
        rest.insert_str(0, replace.as_str());
        if rest.is_empty() {
            return acts;
        }
    }

    Vec::new()
}
