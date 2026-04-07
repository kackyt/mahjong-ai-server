use mahjong_core::components::world::MahjongWorld;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct GameRegistry {
    games: HashMap<usize, MahjongWorld>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRegistry {
    pub fn insert(&mut self, id: usize, world: MahjongWorld) {
        self.games.insert(id, world);
    }

    pub fn get(&self, id: usize) -> Option<&MahjongWorld> {
        self.games.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut MahjongWorld> {
        self.games.get_mut(&id)
    }

    pub fn remove(&mut self, id: usize) -> Option<MahjongWorld> {
        self.games.remove(&id)
    }
}

pub static G_REGISTRY: Lazy<Mutex<GameRegistry>> = Lazy::new(|| Mutex::new(GameRegistry::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_insert_get() {
        let mut reg = GameRegistry::new();
        let world = MahjongWorld::new(4);
        reg.insert(1, world);
        assert!(reg.get(1).is_some());
    }

    #[test]
    fn test_registry_remove() {
        let mut reg = GameRegistry::new();
        let world = MahjongWorld::new(4);
        reg.insert(1, world);
        assert!(reg.remove(1).is_some());
        assert!(reg.get(1).is_none());
    }
}
