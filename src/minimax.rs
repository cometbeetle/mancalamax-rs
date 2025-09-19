use crate::game::{GameState, Mancala, Move, Player};
use std::time::Duration;

pub(crate) type StateEvalFn<const N: usize> = fn(&GameState<N>, player: Player) -> f32;

#[derive(Debug, Clone)]
pub(crate) struct Minimax<const N: usize> {
    max_depth: usize,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    evaluator: StateEvalFn<N>,
    heuristic: StateEvalFn<N>,
}

#[derive(Debug, Clone)]
pub(crate) struct MinimaxBuilder<const N: usize> {
    max_depth: usize,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    evaluator: StateEvalFn<N>,
    heuristic: StateEvalFn<N>,
}

impl<const N: usize> MinimaxBuilder<N> {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
    pub(crate) fn max_time(mut self, time: Duration) -> Self {
        self.max_time = Some(time);
        self
    }
    pub(crate) fn iterative_deepening(mut self, iterative: bool) -> Self {
        self.iterative_deepening = iterative;
        self
    }
    pub(crate) fn evaluator(mut self, e: StateEvalFn<N>) -> Self {
        self.evaluator = e;
        self
    }
    pub(crate) fn heuristic(mut self, h: StateEvalFn<N>) -> Self {
        self.heuristic = h;
        self
    }
    pub(crate) fn build(&self) -> Minimax<N> {
        Minimax {
            max_depth: self.max_depth,
            max_time: self.max_time,
            iterative_deepening: self.iterative_deepening,
            evaluator: self.evaluator,
            heuristic: self.heuristic,
        }
    }
}

impl MinimaxBuilder<6> {
    pub(crate) fn new_standard() -> Self {
        Self::default()
    }
}

impl<const N: usize> Default for MinimaxBuilder<N> {
    fn default() -> Self {
        let evaluator = |s: &GameState<N>, p: Player| match p {
            Player::One => (s.score(Player::One) - s.score(Player::Two)) as f32,
            Player::Two => (s.score(Player::Two) - s.score(Player::One)) as f32,
        };
        let heuristic = evaluator;
        Self {
            max_depth: 12,
            max_time: None,
            iterative_deepening: false,
            evaluator,
            heuristic,
        }
    }
}

impl<const N: usize> Minimax<N> {
    pub(crate) fn max_depth(&self) -> usize {
        self.max_depth
    }
    pub(crate) fn max_time(&self) -> Option<Duration> {
        self.max_time
    }
    pub(crate) fn iterative_deepening(&self) -> bool {
        self.iterative_deepening
    }
    pub(crate) fn call_evaluator(&self, state: &GameState<N>, player: Player) -> f32 {
        (self.evaluator)(state, player)
    }
    pub(crate) fn call_heuristic(&self, state: &GameState<N>, player: Player) -> f32 {
        (self.heuristic)(state, player)
    }
    pub(crate) fn search() -> Move {
        todo!()
    }
    fn max_value() -> (Move, f32) {
        todo!()
    }
    fn min_value() -> (Move, f32) {
        todo!()
    }
}
