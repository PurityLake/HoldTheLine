use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Waiting,
    MainMenu,
    TransitionToGamePlay,
    GamePlay,
    Pause,
    GameOver,
    Cleanup,
}

impl GameState {
    pub fn transition(&self) -> Self {
        match self {
            GameState::Loading => GameState::Waiting,
            GameState::Waiting => GameState::MainMenu,
            GameState::MainMenu => GameState::TransitionToGamePlay,
            GameState::TransitionToGamePlay => GameState::GamePlay,
            GameState::GamePlay => GameState::Pause,
            GameState::Pause => GameState::GamePlay,
            GameState::GameOver => GameState::Cleanup,
            GameState::Cleanup => GameState::MainMenu,
        }
    }
}
