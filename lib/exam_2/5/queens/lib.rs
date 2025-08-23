#derive(Debug)]
pub struct ChessPosition {
    pub rank: i32,
    pub file: i32,
}

#[derive(Debug)]
pub struct Queen {
    pub position: ChessPosition,
}

impl ChessPosition {
    pub fn new(rank: i32, file: i32) -> Option<Self> {}
}

impl Queen {
    pub fn new(position: ChessPosition) -> Self {}

    pub fn can_attack(&self, other: &Queen) -> bool {}
}
