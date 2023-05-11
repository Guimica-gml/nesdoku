use rand::Rng;
use rand::seq::SliceRandom;
use std::io;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub enum CellValue {
    Certain(u32),
    Uncertain(Vec<u32>),
}

impl CellValue {
    pub fn is_certain(&self) -> bool {
        matches!(*self, CellValue::Certain(_))
    }

    pub fn as_vec(&self) -> Vec<u32> {
        match self {
            CellValue::Certain(num) => vec![*num],
            CellValue::Uncertain(nums) => nums.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    value: CellValue,
    is_static: bool,
}

impl Cell {
    pub fn value(&self) -> &CellValue { &self.value }
    pub fn is_static(&self) -> bool { self.is_static }

    pub fn new(value: CellValue, is_static: bool) -> Self {
        Self { value, is_static }
    }
}

#[derive(Debug, Clone)]
pub struct Sudoku {
    // Will always be BOARD_DIM x BOARD_DIM
    board: Vec<Vec<Cell>>,
}

impl Sudoku {
    pub const BOARD_DIM: usize = 9;
    pub const QUADRANT_DIM: usize = 3;

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.board[y][x]
    }

    pub fn from_file(filepath: &str) -> io::Result<Self> {
        let board_text = read_to_string(filepath)?;
        let mut board = vec![vec![Cell::new(CellValue::Uncertain(vec![]), false); Self::BOARD_DIM]; Self::BOARD_DIM];

        for (y, line) in board_text.lines().enumerate() {
            for (x, char) in line.chars().enumerate() {
                match char.to_string().parse::<u32>() {
                    Ok(num) => {
                        board[y][x].value = CellValue::Certain(num);
                        board[y][x].is_static = true;
                    }
                    Err(_) => {}
                }
            }
        }

        Ok(Self { board })
    }

    pub fn quadrant_coords(quadrant_x: usize, quadrant_y: usize) -> Vec<(usize, usize)> {
        assert!(quadrant_x < Self::QUADRANT_DIM && quadrant_y < Self::QUADRANT_DIM);

        let quadrant_x = quadrant_x * Self::QUADRANT_DIM;
        let quadrant_y = quadrant_y * Self::QUADRANT_DIM;
        let mut quadrant = vec![];

        for y in quadrant_y..quadrant_y + Self::QUADRANT_DIM {
            for x in quadrant_x..quadrant_x + Self::QUADRANT_DIM {
                quadrant.push((x, y));
            }
        }

        quadrant
    }

    pub fn row_coords(row_index: usize) -> Vec<(usize, usize)> {
        assert!(row_index < Self::BOARD_DIM);
        let mut row = vec![];

        for y in 0..Self::BOARD_DIM {
            row.push((row_index, y));
        }

        row
    }

    pub fn column_coords(column_index: usize) -> Vec<(usize, usize)> {
        assert!(column_index < Self::BOARD_DIM);
        let mut column = vec![];

        for x in 0..Self::BOARD_DIM {
            column.push((x, column_index));
        }

        column
    }

    pub fn update_possible_cell_values(&mut self, x: usize, y: usize) {
        if self.board[y][x].is_static || self.board[y][x].value.is_certain() {
            return;
        }

        let mut possible_values: Vec<u32> = (1..=9).collect();
        let quadrant = Sudoku::quadrant_coords(x / Self::QUADRANT_DIM, y / Self::QUADRANT_DIM);
        let row = Sudoku::row_coords(x);
        let column = Sudoku::column_coords(y);

        for (cx, cy) in quadrant.into_iter().chain(row).chain(column) {
            if let CellValue::Certain(num) = self.board[cy][cx].value {
                if let Some(index) = possible_values.iter().position(|x| *x == num) {
                    possible_values.remove(index);
                }
            }
        }

        self.board[y][x].value = CellValue::Uncertain(possible_values);
    }

    pub fn update_possible_values(&mut self) {
        for y in 0..Self::BOARD_DIM {
            for x in 0..Self::BOARD_DIM {
                self.update_possible_cell_values(x, y);
            }
        }
    }

    pub fn collapse_cell(&mut self, x: usize, y: usize) -> Result<Vec<Sudoku>, String> {
        match &self.board[y][x].value {
            CellValue::Uncertain(numbers) => {
                let mut possible_boards = vec![];
                let nums = numbers.clone();

                if nums.is_empty() {
                    return Err("Cannot collapse cell with no numbers".to_string());
                }

                let mut rng = rand::thread_rng();
                let rand_idx = rng.gen_range(0..nums.len());

                self.board[y][x].value = CellValue::Certain(nums[rand_idx]);

                for i in 0..nums.len() {
                    if i == rand_idx {
                        continue;
                    }

                    let mut sudoku_clone = self.clone();
                    sudoku_clone.board[y][x].value = CellValue::Certain(nums[i]);
                    possible_boards.push(sudoku_clone);
                }

                possible_boards.shuffle(&mut rng);
                Ok(possible_boards)
            }
            CellValue::Certain(_) => Err("Trying to collapse cell with `Certain` value".to_string()),
        }
    }

    pub fn find_less_entropy(&self) -> (usize, usize) {
        let mut index = (0, 0);
        let mut less_entropy = usize::MAX;

        for y in 0..Self::BOARD_DIM {
            for x in 0..Self::BOARD_DIM {
                match &self.board[y][x].value {
                    CellValue::Uncertain(numbers) => {
                        if numbers.len() < less_entropy {
                            index = (x, y);
                            less_entropy = numbers.len();
                        }
                    }
                    CellValue::Certain(_) => {}
                }
            }
        }

        index
    }

    pub fn reset_board(&mut self) {
        for y in 0..Self::BOARD_DIM {
            for x in 0..Self::BOARD_DIM {
                if self.board[y][x].is_static {
                    continue;
                }
                self.board[y][x].value = CellValue::Uncertain(vec![]);
            }
        }
    }

    // TODO: check for duplicate numbers also just refactor this shit altogether
    // It's not possible to have duplicates since the wave function collapse should avoid that
    // but would be good practice to have
    pub fn complete(&self) -> bool {
        let expected_sum = 45;

        // This needs to be checked ahead of time
        for y in 0..Self::BOARD_DIM {
            for x in 0..Self::BOARD_DIM {
                if !self.board[y][x].value.is_certain() {
                    return false;
                }
            }
        }

        // Check sum of all quadrants
        for qy in 0..Self::QUADRANT_DIM {
            for qx in 0..Self::QUADRANT_DIM {
                if Sudoku::quadrant_coords(qx, qy)
                    .into_iter()
                    .map(|(x, y)| self.board[y][x].value.as_vec()[0])
                    .sum::<u32>()
                    != expected_sum
                {
                    return false;
                }
            }
        }

        for i in 0..Self::BOARD_DIM {
            // Check sum of all rows
            if Sudoku::row_coords(i)
                .into_iter()
                .map(|(x, y)| self.board[y][x].value.as_vec()[0])
                .sum::<u32>()
                != expected_sum
            {
                return false;
            }

            // Check sum of all columns
            if Sudoku::column_coords(i)
                .into_iter()
                .map(|(x, y)| self.board[y][x].value.as_vec()[0])
                .sum::<u32>()
                != expected_sum
            {
                return false;
            }
        }

        true
    }
}
