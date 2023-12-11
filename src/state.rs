use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    TransitionToGamePlay,
    GamePlay,
    Pause,
}
