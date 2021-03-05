use core::mem;

use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;

// use rtt_target::rprintln;

pub struct Matrix<R, C> {
    rows: [R; 4],
    columns: [C; 4],

    matrix: [[bool; 4]; 4],
    previous_matrix: [[bool; 4]; 4],
    states: [[KeyState; 4]; 4],
    changed_states: [[bool; 4]; 4]
}

#[derive(Clone, Copy)]
pub enum KeyState {
    Pressed,
    Released,
    Pressing,
    Releasing
}

pub struct Changes<'a> {
    matrix_y: usize,
    matrix_x: usize,

    states: &'a[[KeyState; 4]; 4],
    changed_states: &'a mut[[bool; 4]; 4]
}

pub struct Change {
    pub matrix_y: usize,
    pub matrix_x: usize,
    pub new_state: KeyState
}

impl<'a> Iterator for Changes<'a> {
    type Item = Change;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
       for y in self.matrix_y..4 {
           for x in self.matrix_x..4 {
                if self.changed_states[y][x] {

                    self.matrix_x = x + 1;
                    self.matrix_y = y;
                    self.changed_states[y][x] = false;

                    return Some(Change {
                       new_state: self.states[y][x],
                       matrix_x: x,
                       matrix_y: y
                    })
                }
           }
       }
       None
    }
}

impl<R, C> Matrix<R, C> 
where
    R: InputPin<>,
    C: OutputPin<> {

    pub fn new(rows: [R; 4], columns: [C; 4]) -> Matrix<R, C> {
        // pub fn new(rows: [PB<Input<PullUp>>; 4], columns: [PA<Output<PushPull>>; 4]) -> Matrix {
        Matrix{
            columns, rows,
            matrix:[[false; 4]; 4],
            previous_matrix: [[false; 4]; 4],
            states: [[KeyState::Released; 4]; 4],
            changed_states: [[false; 4]; 4]
        }
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> &[[KeyState; 4]; 4] {
        &self.states
    }

    pub fn changes(&mut self) -> Changes {
        Changes{
            matrix_y: 0, matrix_x: 0,
            states: &self.states,
            changed_states: &mut self.changed_states
        }
    }

    pub fn update(&mut self) {
        let rows = &self.rows;
        let columns = &mut self.columns;

        for (i_col, col) in columns.iter_mut().enumerate() {
            match col.set_low() { Ok(_) => (), Err(_) => panic!("Set Pin") }

            for (i_row, row) in rows.iter().enumerate() {
                self.matrix[i_col][i_row] = match row.is_low() { 
                    Ok(is_low) => is_low, 
                    Err(_) => panic!("Could not read pin") 
                };
                
                if self.matrix[i_col][i_row] != self.previous_matrix[i_col][i_row] {
                    if self.matrix[i_col][i_row] {
                        self.states[i_col][i_row] = KeyState::Pressing;
                        self.changed_states[i_col][i_row] = true;
                    } else {
                        self.states[i_col][i_row] = KeyState::Releasing;
                        self.changed_states[i_col][i_row] = true;
                    }
                } else {
                    match self.states[i_col][i_row] {
                        KeyState::Pressing => { 
                            self.states[i_col][i_row] = KeyState::Pressed;
                            self.changed_states[i_col][i_row] = true;
                        },
                        KeyState::Releasing => {
                            self.changed_states[i_col][i_row] = true;
                            self.states[i_col][i_row] = KeyState::Released;
                        },
                        _ => ()
                    }
                }
            }

            match col.set_high() { Ok(_) => (), Err(_) => panic!("Set Pin") }
        }

        // rprintln!("{:?}", self.matrix);
        mem::swap(&mut self.matrix, &mut self.previous_matrix);
    }
} 