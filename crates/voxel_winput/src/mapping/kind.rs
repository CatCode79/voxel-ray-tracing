//= INPUT KIND ENUM ==========================================================

#[derive(Clone, Copy, Debug)]
pub enum InputKind {
    GetVoxel,
    PutVoxel,
    InventoryPrev,
    InventoryNext,
    WalkForward,
    WalkLeft,
    WalkBackward,
    WalkRight,
    Jump,
    SlowPace,
    Flying,
    Max,
}
