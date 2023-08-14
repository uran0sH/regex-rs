mod nfa;

#[cfg(test)]
mod tests {
    use crate::nfa;

    #[test]
    pub fn test_match() {
        {
            let pattern = "a+b+";
            let graph = nfa::NFAGraph::new(pattern);
            assert_eq!(graph.is_match("aaaabbb"), true);
            let pattern = "a(b|c)*";
            let graph = nfa::NFAGraph::new(pattern);
            assert_eq!(graph.is_match("abbcbbcc"), true);
            assert_eq!(graph.is_match("bcbbcc"), false);
        }
    }
}
