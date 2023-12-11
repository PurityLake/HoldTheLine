use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    TransitionToGamePlay,
    GamePlay,
    Pause,
}

impl GameState {
    pub fn transition(&self) -> Self {
        match self {
            GameState::MainMenu => GameState::TransitionToGamePlay,
            GameState::TransitionToGamePlay => GameState::GamePlay,
            GameState::GamePlay => GameState::Pause,
            GameState::Pause => GameState::GamePlay,
        }
    }
}
