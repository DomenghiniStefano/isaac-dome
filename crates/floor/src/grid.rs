/// The level grid is 13 wide and 13 tall; a cell's index is `y * WIDTH + x`.
pub const WIDTH: u16 = 13;
pub const HEIGHT: u16 = 13;
pub const CELLS: usize = (WIDTH * HEIGHT) as usize;

/// Where the game says a run starts: `CURRENT ROOM INDEX 84`, printed once per floor.
pub const START: u16 = 84;

/// The cells that touch `cell`, orthogonally. Never wraps a row, and answers nothing for an
/// index outside the grid rather than clamping it into a plausible wrong cell.
pub fn neighbours(cell: u16) -> Vec<u16> {
    if cell as usize >= CELLS {
        return Vec::new();
    }
    let (x, y) = (cell % WIDTH, cell / WIDTH);
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push(cell - 1);
    }
    if x + 1 < WIDTH {
        out.push(cell + 1);
    }
    if y > 0 {
        out.push(cell - WIDTH);
    }
    if y + 1 < HEIGHT {
        out.push(cell + WIDTH);
    }
    out
}
