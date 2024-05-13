use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use Value::*;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    // None indicates a fresh turn
    pub unreserved: Option<Vec<Value>>,
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

pub fn step<'a, F>(state: State, strategy: F) -> State
where
    F: Fn(&State) -> Vec<Value>,
{
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

fn _step<'a, F, R>(state: State, strategy: F, roll: R) -> State
where
    F: Fn(&State) -> Vec<Value>,
    R: Fn() -> Value,
{
    let rolled: Vec<Value> = state
        .unreserved
        // TODO does this roll each time? or just copy one value 6 times
        .unwrap_or_else(|| vec![roll(); 6])
        .iter()
        .map(|_| roll())
        .collect();

    let max_score = score(&rolled);
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

#[test]
fn test_step() {
    let next = _step(
        State::default(),
        |st| st.unreserved.clone().unwrap_or(vec![]),
        || One,
    );
    assert_eq!(Some(vec![]), next.unreserved);
    assert_eq!(6000, next.score_at_risk);
}

// used to score six OR LESS dice
pub fn score<'a>(dice: &[Value]) -> usize {
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

        // straight
        if let Some(6) = m.get(&1).map(|xs| xs.len()) {
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
            m.retain(|_, values| !values.contains(&Five));
            return fives * 50 + score(m);
        }

        0
    }

    let count = |v: Value| dice.iter().filter(|&vv| *vv == v).count();
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

#[test]
fn test_scores() {
    assert_eq!(3000, score(&[One, One, One, One, One, One]));
    assert_eq!(2500, score(&[One, Two, Three, Four, Five, Six]));
    assert_eq!(2050, score(&[One, One, One, One, One, Five]));
    assert_eq!(2500, score(&[One, One, One, Two, Two, Two]));
    assert_eq!(1500, score(&[One, One, Two, Two, Two, Two]));
    assert_eq!(200, score(&[Five, Five, One, Two, Three, Two]));
    assert_eq!(300, score(&[One, One, One]));
    assert_eq!(0, score(&[]));
    assert_eq!(50, score(&[Two, Five]));
}
