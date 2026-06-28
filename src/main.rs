/// Module containing the State and StateId types for representing automaton states.
pub mod state;

/// Module containing the LevenshteinAutomaton type for building and querying the DFA.
pub mod automaton;
/// Module containing the Matcher type for fuzzy matching against a dictionary.
mod matcher;

/// Module containing the Trie data structure for efficient word storage and traversal.
pub mod utilities;

fn main() {
    println!("Hello, world!");
}