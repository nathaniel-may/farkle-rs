use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use Value::*;

// empty indicates stopping
// requiring a reference to be returned enforces that the return values have to be chosen from the state input itself
pub type Strategy<'a> = fn(&State) -> Vec<&'a Die>;
pub type Roll = dyn Fn() -> Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Die {
    id: &'static str,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    // None indicates a fresh turn
    pub unreserved: Option<Vec<Die>>,
    pub score_at_risk: usize,
    pub score: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            unreserved: None,
            score_at_risk: 0,
            score: 0,
        }
    }
}

// Dice cannot be created outside this library. These are the 6 distinct dice needed to use the library.
pub fn dice() -> [Die; 6] {
    [
        Die {
            id: "1",
            value: One,
        },
        Die {
            id: "2",
            value: One,
        },
        Die {
            id: "3",
            value: One,
        },
        Die {
            id: "4",
            value: One,
        },
        Die {
            id: "5",
            value: One,
        },
        Die {
            id: "6",
            value: One,
        },
    ]
}

pub fn step(state: State, strategy: Strategy) -> State {
    fn f() -> Value {
        let mut rng = thread_rng();
        let dist = Uniform::new(1, 6);
        match rng.sample(dist) {
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            _ => Six,
        }
    }
    _step(state, strategy, f)
}

fn _step<F>(state: State, strategy: Strategy, roll: F) -> State
where
    F: Fn() -> Value,
{
    let rolled: Vec<Die> = state
        .unreserved
        .unwrap_or_else(|| dice().into())
        .iter()
        .map(|d| Die {
            id: d.id,
            value: roll(),
        })
        .collect();

    let max_score = score(&rolled.iter().collect::<Vec<&Die>>());
    let farkle = max_score == 0;

    if farkle {
        return State {
            unreserved: None,
            score_at_risk: 0,
            score: state.score,
        };
    }

    // state the "player" sees to apply their strategy to
    let mut state = State {
        unreserved: Some(rolled.clone()),
        score_at_risk: state.score_at_risk + max_score,
        score: state.score,
    };
    match &strategy(&state)[..] {
        // strategy didn't reserve any dice, so they cannot continue rolling.
        // score the score at risk and move on
        [] => {
            state.unreserved = None;
            state.score_at_risk = 0;
            state.score = state.score + state.score_at_risk;
        }

        // legality of move is enforced via the type for defining a strategy,
        // so don't check it again here.
        reserved => {
            state.unreserved = Some(
                rolled
                    .into_iter()
                    .filter(|die| !reserved.contains(&die))
                    .collect(),
            );
            state.score_at_risk = state.score_at_risk + score(&reserved);
        }
    }
    state
}

// used to score six OR LESS dice
pub fn score<'a>(dice: &[&Die]) -> usize {
    fn score(m: &mut HashMap<usize, Vec<Value>>) -> usize {
        // base case
        if m.is_empty() {
            return 0;
        }

        // 6 of a kind
        if m.get(&6).is_some() {
            return 3000;
        }

        // two triples
        if let Some(2) = m.get(&3).map(|xs| xs.len()) {
            return 2500;
        }

        // 5 of a kind
        if m.get(&5).is_some() {
            m.remove(&5);
            // the last die could be a 1 or a 5
            return 2000 + score(m);
        }

        // three pairs that are different
        if let Some(3) = m.get(&2).map(|xs| xs.len()) {
            return 1500;
        }

        // four of a kind and one pair = three pairs
        if m.get(&4).is_some() && m.get(&2).is_some() {
            return 1500;
        }

        // four of a kind that is not three pairs
        if m.get(&4).is_some() {
            m.remove(&4);
            // the last die could be a 1 or a 5
            return 2000 + score(m);
        }

        // 3 of a kind
        if let Some(value) = m.get(&3) {
            // more than one triple is caught earlier so this is safe
            let triple_value = match value[0] {
                One | Three => 300,
                Two => 200,
                Four => 400,
                Five => 500,
                Six => 600,
            };
            m.remove(&3);
            // the last die could be a 1 or a 5
            return triple_value + score(m);
        }

        // TODO I can probably figure out how to get rid of this clone
        let ones = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&One) {
                Some(count)
            } else {
                None
            }
        });

        if let Some(ones) = ones {
            m.retain(|_, values| !values.contains(&One));
            return ones * 100 + score(m);
        }

        // TODO I can probably figure out how to get rid of this clone
        let fives = m.clone().into_iter().find_map(|(count, values)| {
            if values.contains(&Five) {
                Some(count)
            } else {
                None
            }
        });

        if let Some(fives) = fives {
            m.retain(|_, values| !values.contains(&One));
            return fives * 50 + score(m);
        }

        0
    }

    let count = |v: Value| dice.iter().filter(|d| d.value == v).count();
    let mut m = HashMap::new();
    for value in [One, Two, Three, Four, Five, Six] {
        let count = count(value);
        // TODO there's probably a cleaner way to write this line
        let mut values: Vec<Value> = m.get(&count).unwrap_or(&vec![]).iter().cloned().collect();
        values.push(value);
        m.insert(count, values);
    }
    score(&mut m)
}
