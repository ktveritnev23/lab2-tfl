use fancy_regex::Regex;
// fancy_regex crate is needed for lookaround
use rand::seq::SliceRandom;
use rand::{Rng, rng};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};

mod automata;
use crate::automata::{AFA, DFA, NFA};

pub fn read_regex(filename: &str) -> Result<String, io::Error> {
    let regex_string = fs::read_to_string(filename)?;
    let parser: Value = serde_json::from_str(regex_string.as_str())?;
    let real_data = parser["regex"].to_string();
    let pref_strip = 1;
    let suff_strip = real_data.len() - 1;
    let raw_regex = &real_data[pref_strip..suff_strip];
    return Ok(raw_regex.to_string());
}

fn generate_random_strings(
    count: usize,
    min_len: usize,
    max_len: usize,
    alphabet: &str,
) -> Vec<String> {
    assert!(
        min_len <= max_len,
        "Minimum length must be not greater than maximum length"
    );

    let mut _rng = rng();

    let min_char = alphabet.chars().nth(0).unwrap();
    let max_char = alphabet.chars().last().unwrap();

    (0..count)
        .map(|_| {
            let len = _rng.random_range(min_len..=max_len);
            (0..len)
                .map(|_| _rng.random_range(min_char..=max_char))
                .collect()
        })
        .collect()
}

// enriched random strings vector to make fuzz testing more performative
fn generate_with_target_rate(
    count: usize,
    min_len: usize,
    max_len: usize,
    target_rate: f64,
    alphabet: &str,
    regex: &fancy_regex::Regex,
) -> Vec<String> {
    assert!(
        min_len <= max_len,
        "Minimum length must be not greater than maximum length"
    );

    let mut rng = rand::rng();
    let target_matches = (count as f64 * target_rate) as usize;
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();

    let min_char = alphabet.chars().nth(0).unwrap();
    let max_char = alphabet.chars().last().unwrap();

    let batch_size = count;
    let max_batches = 100;
    let mut batch_count = 0;

    while (matching.len() < target_matches || non_matching.len() < count - target_matches)
        && batch_count < max_batches
    {
        let batch: Vec<String> = (0..batch_size)
            .map(|_| {
                let len = rng.random_range(min_len..=max_len);
                (0..len)
                    .map(|_| rng.random_range(min_char..=max_char))
                    .collect()
            })
            .collect();

        for s in batch {
            if regex.is_match(&s).unwrap() {
                if matching.len() < target_matches {
                    matching.push(s);
                }
            } else {
                if non_matching.len() < count - target_matches {
                    non_matching.push(s);
                }
            }
        }
        batch_count += 1;
    }

    let mut result = matching;
    result.extend(non_matching);
    result.truncate(count);

    while result.len() < count {
        let len = rng.random_range(min_len..=max_len);
        let s: String = (0..len)
            .map(|_| rng.random_range(min_char..=max_char))
            .collect();
        result.push(s);
    }

    let mut rng = rand::rng();
    let target_matches = (count as f64 * target_rate) as usize;
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();

    while matching.len() < target_matches || non_matching.len() < count - target_matches {
        let batch: Vec<String> = (0..batch_size)
            .map(|_| {
                let len = rng.random_range(min_len..=max_len);
                (0..len)
                    .map(|_| rng.random_range(min_char..=max_char))
                    .collect()
            })
            .collect();

        for s in batch {
            if regex.is_match(&s).unwrap() {
                if matching.len() < target_matches {
                    matching.push(s);
                }
            } else {
                if non_matching.len() < count - target_matches {
                    non_matching.push(s);
                }
            }
        }
    }

    let mut result = matching;
    result.extend(non_matching);
    result.shuffle(&mut rng);
    result
}

fn main() -> io::Result<()> {
    let alphabet = "ab";

    let regex_path = "files/regex.json";
    let dfa_path = "files/dfa.json";
    let nfa_path = "files/nfa.json";
    let regex_ext_path = "files/regex_ext.json";
    let afa_path = "files/afa.json";

    let regex_string = Regex::new(
        read_regex(regex_path)
            .expect("Failed to extract regex string")
            .as_str(),
    )
    .expect("Failed to initialize regex");
    let ext_regex_string = Regex::new(
        read_regex(regex_ext_path)
            .expect("Failed to extract extended regex string")
            .as_str(),
    )
    .expect("Failed to initialize regex");

    let af = AFA::parse_afa(afa_path);
    let df = DFA::parse_dfa(dfa_path);
    let nf = NFA::parse_nfa(nfa_path);

    // generating target strings with target rate against regex (as normal random generation gets only ~2% matches)
    let test_strings = generate_with_target_rate(1000, 8, 16, 0.2, alphabet, &regex_string);

    let mut success_counter = 0;
    let mut failure_counter = 0;

    fn all_equal(values: &[bool]) -> bool {
        values.windows(2).all(|w| w[0] == w[1])
    }

    struct test_res {
        case: String,
        regex_res: bool,
        dfa_res: bool,
        nfa_res: bool,
        ext_regex_res: bool,
        afa_res: bool,
    }

    let mut succ: Vec<test_res> = vec![];
    let mut fails: Vec<test_res> = vec![];
    let example_limit = 10;

    for string in test_strings.iter() {
        let results = [
            regex_string.is_match(&string).unwrap(),
            DFA::accepts(&df, &string),
            NFA::accepts(&nf, &string),
            ext_regex_string.is_match(&string).unwrap(),
            AFA::accepts(&af, &string),
        ];
        if all_equal(&results) {
            success_counter += 1;
            if success_counter < example_limit {
                succ.push(test_res {
                    case: string.to_string(),
                    regex_res: results[0],
                    dfa_res: results[1],
                    nfa_res: results[2],
                    ext_regex_res: results[3],
                    afa_res: results[4],
                });
            }
        } else {
            failure_counter += 1;
            if failure_counter < example_limit {
                fails.push(test_res {
                    case: string.to_string(),
                    regex_res: results[0],
                    dfa_res: results[1],
                    nfa_res: results[2],
                    ext_regex_res: results[3],
                    afa_res: results[4],
                });
            }
        }
    }

    fn print_test(res: &test_res) {
        println!("String: {}", res.case);
        println!("Regex result: {}", res.regex_res);
        println!("DFA result: {}", res.dfa_res);
        println!("NFA result: {}", res.nfa_res);
        println!("Extended regex result: {}", res.ext_regex_res);
        println!("AFA result: {}", res.afa_res);
    }

    println!("Fuzz testing results:");
    println!("Success: {}", success_counter);
    println!("Failure: {}", failure_counter);
    if !fails.is_empty() {
        println!("Examples of failures:");
        fails.iter().for_each(print_test);
    }
    if !succ.is_empty() {
        println!("Examples of success:");
        succ.iter().for_each(print_test);
    }

    // check if the automata are encoded and read correctly
    print!("Print Graphviz source for automata [y/n]? ");
    std::io::stdout().flush().unwrap();
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read user input");

    let trimmed_str = user_input.trim();

    let yes = ["y", "yes", "д", "да"];

    if yes.iter().any(|&s| s == trimmed_str) {
        let res = DFA::to_graphviz_source(&df);
        println!("{}", res);
        let res_n = NFA::to_graphviz_source(&nf);
        println!("{}", res_n);
        let res_a = AFA::to_graphviz_source(&af);
        println!("{}", res_a);
    }

    Ok(())
}
