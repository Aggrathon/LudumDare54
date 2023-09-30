use bevy::prelude::*;

#[derive(Resource)]
pub struct Level {
    width: usize,
    height: usize,
    index: usize,
    floor: Vec<usize>,
}

impl Level {
    pub fn new(width: usize, height: usize) -> Self {
        Level {
            width,
            height,
            index: 0,
            floor: vec![usize::MAX; width * height],
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, i: usize, j: usize) -> Option<&usize> {
        self.floor.get(j * self.width + i)
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut usize> {
        self.floor.get_mut(j * self.width + i)
    }

    pub fn offset(&self) -> (f32, f32) {
        (-(self.width as f32 * 0.5), -(self.height as f32 * 0.5))
    }

    pub fn next_index(&mut self) -> usize {
        self.index += 1;
        self.index
    }
}
