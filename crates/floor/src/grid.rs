use crate::room::{Cell, RoomKind};

/// The level grid is 13 wide and 13 tall; a cell's index is `y * WIDTH + x`.
pub const WIDTH: u16 = 13;
pub const HEIGHT: u16 = 13;
pub const CELLS: usize = (WIDTH * HEIGHT) as usize;

/// Where the game says a run starts: `CURRENT ROOM INDEX 84`, printed once per floor. Only the
/// tests name it; the crate reads the start from the room the user painted as one.
#[cfg(feature = "test-api")]
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

/// A painted floor: 169 cells, nothing else. It does not know which floor it is, and it does
/// not know whether it is finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    cells: Vec<Cell>,
}

impl Grid {
    pub fn empty() -> Self {
        Self {
            cells: vec![Cell::Empty; CELLS],
        }
    }

    /// `None` when the caller hands over something that is not a grid. Padding it would make
    /// a truncated payload look like a floor with empty cells at the end.
    pub fn from_cells(cells: Vec<Cell>) -> Option<Self> {
        (cells.len() == CELLS).then_some(Self { cells })
    }

    pub fn at(&self, cell: u16) -> Cell {
        self.cells
            .get(cell as usize)
            .copied()
            .unwrap_or(Cell::Empty)
    }

    pub fn set(&mut self, cell: u16, value: Cell) {
        if let Some(slot) = self.cells.get_mut(cell as usize) {
            *slot = value;
        }
    }

    pub fn painted(&self) -> usize {
        self.cells.iter().filter(|c| **c != Cell::Empty).count()
    }

    pub fn neighbour_kinds(&self, cell: u16) -> Vec<RoomKind> {
        neighbours(cell)
            .into_iter()
            .filter_map(|n| match self.at(n) {
                Cell::Empty => None,
                Cell::Room { kind, shape: _ } => Some(kind),
            })
            .collect()
    }
}
