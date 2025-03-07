//= IMPORTS ==================================================================

//use std::slice::Iter;

//= ENUM MOUSE BUTTON ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    //Shift,
    //Control,
    Middle,
    X1,
    X2,
}

/*impl MouseButton {
    pub fn iterator() -> Iter<'static, MouseButton> {
        static DIRECTIONS: [MouseButton; 5] = [
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::X1,
            MouseButton::X2,
        ];
        DIRECTIONS.iter()
    }

    pub fn num2enum<N: num::PrimInt>(number: N) -> Option<MouseButton> {
        match number.to_usize().unwrap() {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Right),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::X1),
            4 => Some(MouseButton::X2),
            _ => None, // TODO: messaggio di errore in debug?
        }
    }
}*/
