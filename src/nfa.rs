use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(usize);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Transition {
    Epsilon,
    Char(Vec<char>),
}

#[derive(Debug, Clone)]
pub struct State {
    pub id: StateId,
    pub outs: HashMap<StateId, Transition>,
}

impl State {
    pub fn new(id: StateId) -> Self {
        Self {
            id,
            outs: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Frag {
    start: StateId,
    end: Vec<StateId>,
}

#[derive(Debug)]
pub struct NFAGraph {
    pub states: HashMap<StateId, State>,
    pub last_id: usize,
    pub start: StateId,
    pub ends: Vec<StateId>,
}

impl NFAGraph {
    pub fn new(pattern: &str) -> Self {
        let post = re2post(pattern);
        match post {
            Some(post) => {
                Self::compile(&post)
            }
            None => panic!("illegal pattern")
        }
    }
    pub fn compile(post: &str) -> Self {
        let mut stack: Vec<Frag> = Vec::new();
        let mut graph = NFAGraph {
            states: HashMap::new(),
            last_id: 0,
            start: StateId(0),
            ends: vec![StateId(0)],
        };
        for post_char in post.chars() {
            match post_char {
                '.' => {
                    if stack.len() < 2 {
                        return graph;
                    }
                    let frag2 = stack.pop().unwrap();
                    let frag1 = stack.pop().unwrap();
                    for next in frag1.end.iter() {
                        let state = graph.states.get_mut(next).unwrap();
                        state.outs.insert(frag2.start, Transition::Epsilon);
                    }
                    stack.push(Frag {
                        start: frag1.start,
                        end: frag2.end,
                    });
                }
                '|' => {
                    if stack.len() < 2 {
                        return graph;
                    }
                    let frag2 = stack.pop().unwrap();
                    let frag1 = stack.pop().unwrap();
                    let mut start = State::new(StateId(graph.last_id));
                    let end = State::new(StateId(graph.last_id + 1));
                    graph.last_id += 2;
                    start.outs.insert(frag1.start, Transition::Epsilon);
                    start.outs.insert(frag2.start, Transition::Epsilon);
                    for next in frag1.end.iter() {
                        let state = graph.states.get_mut(next).unwrap();
                        state.outs.insert(end.id, Transition::Epsilon);
                    }
                    for next in frag2.end.iter() {
                        let state = graph.states.get_mut(next).unwrap();
                        state.outs.insert(end.id, Transition::Epsilon);
                    }
                    graph.states.insert(start.id, start.clone());
                    graph.states.insert(end.id, end.clone());
                    stack.push(Frag {
                        start: start.id,
                        end: vec![end.id],
                    });
                }
                '?' => {
                    if stack.is_empty() {
                        return graph;
                    }
                    let frag = stack.pop().unwrap();
                    let start = graph.states.get_mut(&frag.start).unwrap();
                    frag.end.iter().for_each(|e| {
                        start.outs.insert(*e, Transition::Epsilon);
                    });
                    stack.push(frag);
                }
                '*' => {
                    if stack.is_empty() {
                        return graph;
                    }
                    let frag = stack.pop().unwrap();
                    let mut start = State::new(StateId(graph.last_id));
                    let end = State::new(StateId(graph.last_id + 1));
                    graph.last_id += 2;
                    start.outs.insert(frag.start, Transition::Epsilon);
                    let old_start = graph.states.get_mut(&frag.start).unwrap();
                    for next in frag.end.iter() {
                        old_start.outs.insert(*next, Transition::Epsilon);
                    }
                    for next in frag.end.iter() {
                        let state = graph.states.get_mut(next).unwrap();
                        state.outs.insert(end.id, Transition::Epsilon);
                        state.outs.insert(frag.start, Transition::Epsilon);
                    }
                    graph.states.insert(start.id, start.clone());
                    graph.states.insert(end.id, end.clone());
                    stack.push(Frag {
                        start: start.id,
                        end: vec![end.id],
                    });
                }
                '+' => {
                    if stack.is_empty() {
                        return graph;
                    }
                    let frag = stack.pop().unwrap();
                    let mut start = State::new(StateId(graph.last_id));
                    let end = State::new(StateId(graph.last_id + 1));
                    graph.last_id += 2;
                    start.outs.insert(frag.start, Transition::Epsilon);
                    for next in frag.end.iter() {
                        let state = graph.states.get_mut(next).unwrap();
                        state.outs.insert(end.id, Transition::Epsilon);
                        state.outs.insert(frag.start, Transition::Epsilon);
                    }
                    graph.states.insert(start.id, start.clone());
                    graph.states.insert(end.id, end.clone());
                    stack.push(Frag {
                        start: start.id,
                        end: vec![end.id],
                    });
                }
                c if c.is_alphanumeric() => {
                    let mut start = State::new(StateId(graph.last_id));
                    let end = State::new(StateId(graph.last_id + 1));
                    graph.last_id += 2;
                    start.outs.insert(end.id, Transition::Char(vec![c]));
                    graph.states.insert(start.id, start.clone());
                    graph.states.insert(end.id, end.clone());
                    stack.push(Frag {
                        start: start.id,
                        end: vec![end.id],
                    });
                }
                _ => {
                    panic!("illegal character")
                }
            }
        }
        if !stack.is_empty() {
            let frag = stack.pop().unwrap();
            graph.start = frag.start;
            graph.ends = frag.end;
        }
        graph
    }

    pub fn is_match(&self, s: &str) -> bool {
        self.check_match(s, self.start)
    }

    fn check_match(&self, s: &str, state_id: StateId) -> bool {
        let mut current_set = vec![state_id];
        let mut next_set = self.closure(current_set);
        for (i, c) in s.chars().enumerate() {
            current_set = self.move2(c, &next_set);
            next_set = self.closure(current_set);

            if next_set.is_empty() {
                return false;
            }

            for state_id in next_set.iter() {
                let state = self.states.get(state_id).unwrap();
                if state.outs.is_empty() && i == s.len() - 1 {
                    return true;
                }
            }
        }
        false
    }

    fn closure(&self, current_set: Vec<StateId>) -> Vec<StateId> {
        let mut closure_set = current_set.clone();
        let mut queue = VecDeque::new();
        for cl in current_set {
            queue.push_back(cl);
        }
        while !queue.is_empty() {
            let state_id = queue.pop_front().unwrap();
            let state = self.states.get(&state_id).unwrap();
            for out in state.outs.iter() {
                if let Transition::Epsilon = out.1 {
                    if !closure_set.contains(out.0) {
                        closure_set.push(*out.0);
                        queue.push_back(*out.0);
                    }
                }
            }
        }
        closure_set
    }

    fn move2(&self, c: char, current_set: &[StateId]) -> Vec<StateId> {
        let mut next_set = Vec::new();
        for state_id in current_set.iter() {
            let state = self.states.get(state_id).unwrap();
            for out in state.outs.iter() {
                if let Transition::Char(chars) = out.1 {
                    if chars.contains(&c) {
                        next_set.push(*out.0);
                    }
                }
            }
        }
        next_set
    }

    pub fn display(&self) {
        for state in self.states.iter() {
            println!("state id: {:?}, state outs: {:?}", state.0 .0, state.1.outs)
        }
    }
}

pub fn re2post(re: &str) -> Option<String> {
    let mut postfix: String = String::new();
    struct Paren {
        natom: usize,
        nalt: usize,
    }
    let mut paren: Vec<Paren> = Vec::new();
    let mut natom = 0usize;
    let mut nalt = 0usize;
    for re_char in re.chars() {
        match re_char {
            '(' => {
                if natom > 1 {
                    natom -= 1;
                    postfix.push('.');
                }
                paren.push(Paren { natom, nalt });
                natom = 0;
                nalt = 0;
            }
            '|' => {
                nalt += 1;
                if natom == 0 {
                    return None;
                }
                while natom > 1 {
                    natom -= 1;
                    postfix.push('.');
                }
                if natom == 1 {
                    natom = 0;
                }
            }
            ')' => {
                if paren.is_empty() {
                    return None;
                }
                if natom == 0 {
                    return None;
                }
                while natom > 1 {
                    natom -= 1;
                    postfix.push('.');
                }
                while nalt > 0 {
                    nalt -= 1;
                    postfix.push('|');
                }
                let p = paren.pop().unwrap();
                natom = p.natom + 1;
                nalt = p.nalt;
            }
            '*' | '+' | '?' => {
                if natom == 0 {
                    return None;
                }
                postfix.push(re_char);
            }
            c if c.is_alphanumeric() => {
                if natom > 1 {
                    natom -= 1;
                    postfix.push('.');
                }
                postfix.push(c);
                natom += 1;
            }
            _ => {
                panic!("illegal character")
            }
        }
    }
    // Parentheses do not come in pairs. It's an error.
    if !paren.is_empty() {
        return None;
    }
    while natom > 1 {
        natom -= 1;
        postfix.push('.');
    }
    while nalt > 0 {
        nalt -= 1;
        postfix.push('|');
    }
    Some(postfix)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::nfa::StateId;

    #[test]
    fn test_re_2_post() {
        assert_eq!("a+b+.", super::re2post("a+b+").unwrap_or_default());
        assert_eq!(
            "azd.c.e||+b+.",
            super::re2post("(a|zdc|e)+b+").unwrap_or_default()
        );
        assert_eq!(
            "azd*.c+.e||+b+.",
            super::re2post("(a|zd*c+|e)+b+").unwrap_or_default()
        );
    }

    #[test]
    pub fn test_nfa() {
        let pattern = "a+b+";
        let post = super::re2post(pattern).unwrap_or_default();
        assert_eq!("a+b+.", post);
        let graph = super::NFAGraph::compile(&post);
        assert_eq!(graph.states.len(), 8);
        graph.display();
        println!(
            "state {} next is {:?}",
            3,
            graph.states.get(&StateId(3)).unwrap().outs
        );
        {
            let mut map = HashMap::new();
            map.insert(StateId(0), super::Transition::Epsilon);
            map.insert(StateId(3), super::Transition::Epsilon);
            assert_eq!(map, graph.states.get(&StateId(1)).unwrap().outs);
        }
        {
            let mut map = HashMap::new();
            map.insert(StateId(4), super::Transition::Epsilon);
            map.insert(StateId(7), super::Transition::Epsilon);
            assert_eq!(map, graph.states.get(&StateId(5)).unwrap().outs);
        }

        let pattern = "a(b|c)*";
        let post = super::re2post(pattern).unwrap_or_default();
        assert_eq!("abc|*.", post);
        let graph = super::NFAGraph::compile(&post);
        graph.display();
        {
            let mut map = HashMap::new();
            map.insert(StateId(6), super::Transition::Epsilon);
            map.insert(StateId(9), super::Transition::Epsilon);
            assert_eq!(map, graph.states.get(&StateId(7)).unwrap().outs);
        }
        {
            let mut map = HashMap::new();
            map.insert(StateId(4), super::Transition::Epsilon);
            map.insert(StateId(2), super::Transition::Epsilon);
            map.insert(StateId(7), super::Transition::Epsilon);
            assert_eq!(map, graph.states.get(&StateId(6)).unwrap().outs);
        }
        assert_eq!(graph.states.len(), 10);
    }

    #[test]
    pub fn test_match() {
        {
            let pattern = "a+b+";
            let post = super::re2post(pattern).unwrap_or_default();
            let graph = super::NFAGraph::compile(&post);
            assert_eq!(graph.is_match("aaaabbb"), true);
            let pattern = "a(b|c)*";
            let post = super::re2post(pattern).unwrap_or_default();
            let graph = super::NFAGraph::compile(&post);
            assert_eq!(graph.is_match("abbcbbcc"), true);
            assert_eq!(graph.is_match("bcbbcc"), false);
        }
    }
}
