use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayUs;

pub struct Matrix<R, C> {
    rows: [R; 4],
    columns: [C; 4],

    states: [[KeyState; 4]; 4],
    states_changed: [[bool; 4]; 4]
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
    states_changed: &'a mut[[bool; 4]; 4]
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
                if self.states_changed[y][x] {

                    self.matrix_x = x + 1;
                    self.matrix_y = y;
                    self.states_changed[y][x] = false;

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
        Matrix{
            columns, rows,
            states: [[KeyState::Released; 4]; 4],
            states_changed: [[false; 4]; 4]
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
            states_changed: &mut self.states_changed
        }
    }

    pub fn update<DELAY: DelayUs<u16>> (&mut self, delay: &mut DELAY) {
        let rows = &self.rows;
        let columns = &mut self.columns;

        for (i_col, col) in columns.iter_mut().enumerate() {
            col.set_low().map_err(| _ | ()).unwrap();

            // need some time to read right input (maybe capacitances)
            delay.delay_us(5);

            for (i_row, row) in rows.iter().enumerate() {
                let button = row.is_low().map_err(| _ | ()).unwrap();
                
                match self.states[i_col][i_row] {
                    KeyState::Pressed if !button => { self.states[i_col][i_row] = KeyState::Releasing; self.states_changed[i_col][i_row] = true},
                    KeyState::Released if button => { self.states[i_col][i_row] = KeyState::Pressing; self.states_changed[i_col][i_row] = true},
                    KeyState::Releasing if button => { self.states[i_col][i_row] = KeyState::Pressing; self.states_changed[i_col][i_row] = true},
                    KeyState::Releasing if !button => { self.states[i_col][i_row] = KeyState::Released; self.states_changed[i_col][i_row] = true},
                    KeyState::Pressing if button => { self.states[i_col][i_row] = KeyState::Pressed; self.states_changed[i_col][i_row] = true},
                    KeyState::Pressing if !button => { self.states[i_col][i_row] = KeyState::Releasing; self.states_changed[i_col][i_row] = true},
                    _ => ()
                }
            }

            col.set_high().map_err(| _ | ()).unwrap();
        }
    }
} 