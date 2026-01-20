use std::fs::File;
use serde_json::{self, Value};
use std::collections::{HashMap, HashSet};
use std::io::BufReader;

pub struct DFA {
    pub alphabet: Vec<char>,
    pub states: Vec<usize>,
    pub transition: HashMap<(usize, char), usize>,
    pub start_state: usize,
    pub final_states: Vec<usize>,
}

impl DFA {
    pub fn parse_dfa(filename: &str) -> Self {
        let marker_array = [
            "alphabet",
            "states_min",
            "states_max",
            "final_states",
            "transitions",
        ];

        let file = File::open(filename).expect("Failed to open DFA JSON file");
        let reader = BufReader::new(file);
        let raw_data: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

        let alphabet_str = raw_data[marker_array[0]]
            .as_str()
            .expect("Alphabet should be a string");
        let alphabet: Vec<char> = alphabet_str.chars().collect();

        let states_min = raw_data[marker_array[1]]
            .as_u64()
            .expect("Minimum state should be a number") as usize;
        let states_max = raw_data[marker_array[2]]
            .as_u64()
            .expect("Maximum state should be a number") as usize;

        assert!(
            states_min <= states_max,
            "Minimum state should be not larger than maximum state"
        );

        let states: Vec<usize> = (states_min..=states_max).collect();

        let start_state = states_min;

        let final_states_value = &raw_data[marker_array[3]];
        let final_states: Vec<usize> = final_states_value
            .as_array()
            .expect("Final states should be an array")
            .iter()
            .map(|v| v.as_u64().expect("Final state should be a number") as usize)
            .collect();

        let transitions_value = &raw_data[marker_array[4]];
        let transition: HashMap<(usize, char), usize> = transitions_value
            .as_array()
            .expect("Transtion function should be in an array")
            .iter()
            .map(|t| {
                let trans_row = t.as_array().expect("Transition should be an array");
                let source_state = trans_row[0]
                    .as_u64()
                    .expect("Source state should be a number") as usize;
                let input_symbol = trans_row[1]
                    .as_str()
                    .expect("Symbol should be of string type")
                    .chars()
                    .next()
                    .expect("Symbol should be a single character");
                let dest_state = trans_row[2]
                    .as_u64()
                    .expect("Destination state should be a number") as usize;
                ((source_state, input_symbol), dest_state)
            })
            .collect();

        Self {
            alphabet,
            states,
            transition,
            start_state,
            final_states,
        }
    }

    pub fn accepts(&self, input: &str) -> bool {
        let mut current_state = self.start_state;

        for ch in input.chars() {
            if !self.alphabet.contains(&ch) {
                return false; // Invalid input symbol
            }

            // Look up transition
            if let Some(&next_state) = self.transition.get(&(current_state, ch)) {
                current_state = next_state;
            } else {
                return false;
            }
        }

        self.final_states.contains(&current_state)
    }

    pub fn to_graphviz_source(&self) -> String {
        let mut gs = String::new();

        gs.push_str("digraph DFA {\n");
        gs.push_str("   rankdir=LR;\n");
        gs.push_str("   node [shape = circle];\n");

        gs.push_str("   start [shape=none, label=\"\"];\n ");
        gs.push_str(&format!("  start -> {};\n", self.start_state));

        if !self.final_states.is_empty() {
            gs.push_str("   node [shape = doublecircle];\n");
            for state in &self.final_states {
                gs.push_str(&format!("   {};\n", state));
            }
            gs.push_str("   node [shape = circle];\n");
        }

        for ((from, char), to) in &self.transition {
            gs.push_str(&format!(
                "   {} -> {} [label=\"{}\"]\n",
                from,
                to,
                char.escape_default()
            ));
        }

        gs.push_str("}\n");
        gs
    }
}

pub struct NFA {
    pub alphabet: Vec<char>,
    pub states: Vec<usize>,
    pub transition: HashMap<(usize, char), HashSet<usize>>,
    pub start_state: usize,
    pub final_states: Vec<usize>,
}

impl NFA {
    pub fn parse_nfa(filename: &str) -> Self {
        let marker_array = [
            "alphabet",
            "states_min",
            "states_max",
            "final_states",
            "transitions",
        ];

        let file = File::open(filename).expect("Failed to open DFA JSON file");
        let reader = BufReader::new(file);
        let raw_data: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

        let alphabet_str = raw_data[marker_array[0]]
            .as_str()
            .expect("Alphabet should be a string");
        let alphabet: Vec<char> = alphabet_str.chars().collect();

        let states_min = raw_data[marker_array[1]]
            .as_u64()
            .expect("Minimum state should be a number") as usize;
        let states_max = raw_data[marker_array[2]]
            .as_u64()
            .expect("Maximum state should be a number") as usize;

        let states: Vec<usize> = (states_min..=states_max).collect();

        assert!(
            states_min <= states_max,
            "Minimum state should be not larger than maximum state"
        );

        let start_state = states_min;

        let final_states_value = &raw_data[marker_array[3]];
        let final_states: Vec<usize> = final_states_value
            .as_array()
            .expect("Final states should be an array")
            .iter()
            .map(|v| v.as_u64().expect("Final state should be a number") as usize)
            .collect();

        let transitions_value = &raw_data[marker_array[4]];
        let transition: HashMap<(usize, char), HashSet<usize>> = transitions_value
            .as_array()
            .expect("Transtion function should be in an array")
            .iter()
            .map(|t| {
                let trans_row = t.as_array().expect("Transition should be an array");
                let source_state = trans_row[0]
                    .as_u64()
                    .expect("Source state should be a number")
                    as usize;
                let input_symbol = trans_row[1]
                    .as_str()
                    .expect("Symbol should be of string type")
                    .chars()
                    .next()
                    .expect("Symbol should be a single character");
                let dest_state: Vec<usize> = trans_row[2]
                    .as_array()
                    .expect("Destination states should be an array")
                    .iter()
                    .map(|v| {
                        v.as_u64()
                            .expect("Every destination state should be a number")
                            as usize
                    })
                    .collect();
                (
                    (source_state, input_symbol),
                    dest_state.into_iter().collect(),
                )
            })
            .collect();

        Self {
            alphabet,
            states,
            transition,
            start_state,
            final_states,
        }
    }

    pub fn accepts(&self, input: &str) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(self.start_state);

        for ch in input.chars() {
            if !self.alphabet.contains(&ch) {
                return false; // Invalid input symbol
            }

            let mut next_states = HashSet::new();
            for &state in &current_states {
                if let Some(transitions) = self.transition.get(&(state, ch)) {
                    next_states.extend(transitions);
                }
            }
            current_states = next_states;
        }

        current_states
            .iter()
            .any(|state| self.final_states.contains(state))
    }

    pub fn to_graphviz_source(&self) -> String {
        let mut gs = String::new();

        gs.push_str("digraph NFA {\n");
        gs.push_str("   rankdir=LR;\n");
        gs.push_str("   node [shape = circle];\n");

        gs.push_str("   start [shape=none, label=\"\"];\n");
        gs.push_str(&format!("   start -> {};\n", self.start_state));

        if !self.final_states.is_empty() {
            gs.push_str("   node [shape = doublecircle];\n");
            for state in &self.final_states {
                gs.push_str(&format!("   {};\n", state));
            }
            gs.push_str("   node [shape = circle];\n");
        }

        // Regroup transition
        let mut edges: HashMap<(usize, usize), Vec<char>> = HashMap::new();

        for ((src, ch), dest_set) in &self.transition {
            for dst in dest_set {
                edges.entry((*src, *dst)).or_default().push(*ch);
            }
        }

        for ((src, dst), mut chars) in edges {
            // Build a label like "a,b,ε"
            chars.sort();
            let label = chars
                .iter()
                .map(|&c| c.to_string())
                .collect::<Vec<String>>()
                .join(",");

            gs.push_str(&format!("   {} -> {} [label=\"{}\"];\n", src, dst, label));
        }

        gs.push_str("}\n");
        gs
    }
}

#[derive(PartialEq, Eq)]
pub enum StateKind {
    Existential, 
    Universal,
}

/*
epsilon transitions are allowed from conjuctions (universal states) to get single branching point
for complicated non-deterministic branches
(but not used)
*/

pub struct AFA {
    pub alphabet: Vec<char>,
    pub states: Vec<usize>,
    pub transition: HashMap<(usize, Option<char>), HashSet<usize>>,
    pub start_state: usize,
    pub final_states: Vec<usize>,
    pub states_type: Vec<StateKind>,
}

impl AFA {
    pub fn parse_afa(filename: &str) -> Self {
        let marker_array = [
            "alphabet",
            "states_min",
            "states_max",
            "existentials",
            "universals",
            "final_states",
            "transitions",
        ];

        // Reading the file
        let file = File::open(filename).expect("Failed to open DFA JSON file");
        let reader = BufReader::new(file);
        let raw_data: Value = serde_json::from_reader(reader).expect("Failed to parse JSON");

        // Reading the alphabet
        let alphabet_str = raw_data[marker_array[0]]
            .as_str()
            .expect("Alphabet should be a string");
        let alphabet: Vec<char> = alphabet_str.chars().collect();

        // Reading the range of states
        let states_min = raw_data[marker_array[1]]
            .as_u64()
            .expect("Minimum state should be a number") as usize;

        let states_max = raw_data[marker_array[2]]
            .as_u64()
            .expect("Maximum state should be a number") as usize;

        let states: Vec<usize> = (states_min..=states_max).collect();

        assert!(
            states_min <= states_max,
            "Minimum state should be not larger than maximum state"
        );

        let mut states_type: Vec<StateKind> = Vec::with_capacity(states.len());

        for _ in states_min..=states_max {
            states_type.push(StateKind::Existential);
        }

        let exist_states_value = &raw_data[marker_array[3]];
        let exist_array: Vec<usize> = exist_states_value
            .as_array()
            .expect("Existential states must be an array")
            .iter()
            .map(|v| v.as_u64().expect("Existential state should be a number") as usize)
            .collect();

        let univ_states_value = &raw_data[marker_array[4]];
        let univ_array: Vec<usize> = univ_states_value
            .as_array()
            .expect("Universal states must be an array")
            .iter()
            .map(|v| v.as_u64().expect("Universal state must be a number") as usize)
            .collect();

        // collecting to universal and existential arrays
        for i in exist_array {
            states_type[i] = StateKind::Existential;
        }
        for i in univ_array {
            states_type[i] = StateKind::Universal;
        }

        let start_state = states_min;

        let final_states_value = &raw_data[marker_array[5]];
        let final_states: Vec<usize> = final_states_value
            .as_array()
            .expect("Final states should be an array")
            .iter()
            .map(|v| v.as_u64().expect("Final state should be a number") as usize)
            .collect();

        let transitions_value = &raw_data[marker_array[6]];
        let transition: HashMap<(usize, Option<char>), HashSet<usize>> = transitions_value
            .as_array()
            .expect("Transtion function should be in an array")
            .iter()
            .map(|t| {
                let trans_row = t.as_array().expect("Transition should be an array");
                let source_state = trans_row[0]
                    .as_u64()
                    .expect("Source state should be a number")
                    as usize;
                let input_symbol = trans_row[1]
                    .as_str()
                    .expect("Symbol should be of string type")
                    .chars()
                    .next();
                let dest_state: Vec<usize> = trans_row[2]
                    .as_array()
                    .expect("Destination states should be an array")
                    .iter()
                    .map(|v| {
                        v.as_u64()
                            .expect("Every destination state should be a number")
                            as usize
                    })
                    .collect();
                (
                    (source_state, input_symbol),
                    dest_state.into_iter().collect(),
                )
            })
            .collect();

        Self {
            alphabet,
            states,
            transition,
            start_state,
            final_states,
            states_type,
        }
    }

    pub fn accepts(&self, input: &str) -> bool {
        let str_symbols: Vec<char> = input.chars().collect();

        let mut memory: HashMap<(usize, usize), bool> = HashMap::new();

        self.accept_from(self.start_state, 0, &str_symbols, &mut memory)
    }

    pub fn accept_from(
        &self,
        state: usize,
        pos: usize,
        input: &[char],
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(&cached) = memo.get(&(state, pos)) {
            return cached;
        }

        let epsilon_result = self.handle_epsilons(state, pos, input, memo);

        if pos == input.len() {
            let result = self.final_states.contains(&state) || epsilon_result;
            memo.insert((state, pos), result);
            return result;
        }

        let sym = input[pos];
        if !self.alphabet.contains(&sym) {
            return false; // invalid symbol, why match more
        }
        
        let succ = self.transition.get(&(state, Some(sym)));
        let eps_succ = self.transition.get(&(state, None));

        let result = match self.states_type[state] {
            StateKind::Existential => {
                let mut found_path = false;

                if let Some(epsilon_set) = eps_succ {
                    found_path = epsilon_set
                        .iter()
                        .any(|&next| self.accept_from(next, pos, input, memo));
                }

                if !found_path {
                    if let Some(set) = succ {
                        found_path = set
                            .iter()
                            .any(|&next| self.accept_from(next, pos + 1, input, memo))
                    };
                }

                found_path
            }

            StateKind::Universal => {
                let mut all_paths_valid = true;

                if let Some(epsilon_set) = eps_succ {
                    if !epsilon_set.is_empty() {
                        all_paths_valid = epsilon_set
                            .iter()
                            .all(|&next| self.accept_from(next, pos, input, memo));
                    }
                }

                if all_paths_valid {
                    if let Some(set) = succ {
                        if !set.is_empty() {
                            all_paths_valid = set
                                .iter()
                                .all(|&next| self.accept_from(next, pos + 1, input, memo));
                        } else {
                            all_paths_valid = false;
                        }
                    } else {
                        all_paths_valid = false;
                    }
                }

                all_paths_valid
            }
        };

        memo.insert((state, pos), result);
        result
    }

    fn handle_epsilons(
        &self,
        state: usize,
        pos: usize,
        input: &[char],
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(epsilon_set) = self.transition.get(&(state, None)) {
            match self.states_type[state] {
                StateKind::Existential => epsilon_set
                    .iter()
                    .any(|&next| self.accept_from(next, pos, input, memo)),
                StateKind::Universal => {
                    !epsilon_set.is_empty()
                        && epsilon_set
                            .iter()
                            .all(|&next| self.accept_from(next, pos, input, memo))
                }
            }
        } else {
            false
        }
    }

    pub fn to_graphviz_source(&self) -> String {
        let mut gs = String::new();

        gs.push_str("digraph AFA {\n");
        gs.push_str("   rankdir=LR;\n");
        gs.push_str("   node [shape=rectangle];\n ");
        for (i, v) in self.states_type.iter().enumerate() {
            if *v == StateKind::Universal {
                gs.push_str(&format!("  {};\n ", i));
            }
        }
        gs.push_str("  node [shape = circle];\n");

        gs.push_str("   start [shape=none, label=\"\"];\n");
        gs.push_str(&format!("   start -> {};\n", self.start_state));

        if !self.final_states.is_empty() {
            gs.push_str("   node [shape = doublecircle];\n");
            for state in &self.final_states {
                gs.push_str(&format!("   {};\n", state));
            }
            gs.push_str("   node [shape = circle];\n");
        }

        // Regroup transition
        let mut edges: HashMap<(usize, usize), Vec<Option<char>>> = HashMap::new();

        for ((src, ch), dest_set) in &self.transition {
            for dst in dest_set {
                edges.entry((*src, *dst)).or_default().push(*ch);
            }
        }

        for ((src, dst), mut chars) in edges {
            chars.sort();
            // Build a label like "a,b,ε"
            let label = chars
                .iter()
                .map(|&c| c.unwrap_or('ε').to_string())
                .collect::<Vec<String>>()
                .join(",");

            gs.push_str(&format!("   {} -> {} [label=\"{}\"];\n", src, dst, label));
        }

        gs.push_str("}\n");
        gs
    }
}
