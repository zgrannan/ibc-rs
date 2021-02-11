use stateright::*;
mod executor;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Slide {
    Down,
    Up,
    Right,
    Left,
}

struct Puzzle(Vec<u8>);
impl Model for Puzzle {
    type State = Vec<u8>;
    type Action = Slide;

    fn init_states(&self) -> Vec<Self::State> {
        vec![self.0.clone()]
    }

    fn actions(&self, _state: &Self::State, actions: &mut Vec<Self::Action>) {
        actions.append(&mut vec![Slide::Down, Slide::Up, Slide::Right, Slide::Left]);
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let empty = last_state.iter().position(|x| *x == 0).unwrap();
        let empty_y = empty / 3;
        let empty_x = empty % 3;
        let maybe_from = match action {
            Slide::Down if empty_y > 0 => Some(empty - 3), // above
            Slide::Up if empty_y < 2 => Some(empty + 3),   // below
            Slide::Right if empty_x > 0 => Some(empty - 1), // left
            Slide::Left if empty_x < 2 => Some(empty + 1), // right
            _ => None,
        };
        maybe_from.map(|from| {
            let mut next_state = last_state.clone();
            next_state[empty] = last_state[from];
            next_state[from] = 0;
            next_state
        })
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![Property::sometimes("solved", |_, state: &Vec<u8>| {
            let solved = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
            state == &solved
        })]
    }
}

#[test]
fn stateright() {
    Puzzle(vec![1, 4, 2, 3, 5, 8, 6, 7, 0])
        .checker()
        .spawn_bfs()
        .join()
        .assert_properties()
}
