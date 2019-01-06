use ncurses::*;
use super::snake::Snake;
use std::time::Duration;
use std::thread;
use rand::prelude::*;

static LEAST_HEIGHT_FOR_GAME: u32 = 20;
static LEAST_WIDTH_FOR_GAME: u32 = 20;
static mut PAUSED: bool = false;
static mut CHEAT_COUNT: u32 = 0;
static CHEAT_WARNING_NUM: u32 = 100;
static CHEAT_WARNING2_NUM: u32 = 500;
static CHEAT_WARNING3_NUM: u32 = 750;
static CHEAT_MAX_NUM: u32 = 1000;

pub struct Game {
    board: Board,
    board_win: WINDOW,
    info_win: WINDOW,
    mesg_win: WINDOW,
    pause_win: WINDOW,
    refresh_time: Duration,
    difficulty: u8,
}

struct Board {
    height: u32,
    width: u32,
    grid: Vec<Vec<Cell>>,
}

impl Board {
    fn new(height: u32, width: u32) -> Board {
        let a_row: Vec<Cell> = vec![Cell::Empty; width as usize];
        Board { 
            height,
            width,
            grid: vec![a_row.clone(); height as usize],
        }
    }

    fn set_cell(&mut self, y: u32, x: u32, cell: Cell) {
        assert!(y < self.height && x < self.width);
        self.grid[y as usize][x as usize] = cell;
    }

    fn get_cell(&self, y: u32, x:u32) -> & Cell {
        assert!(y < self.height && x < self.width);
        & self.grid[y as usize][x as usize]
    }

    fn is_full(&self) -> bool {
        return false; // FIXME: too troublesome to check
    }

    fn generate_apple(&mut self) -> Option<(u32, u32)>{
       let mut rng = thread_rng();
       if self.is_full() {
           return None;
       }
       loop { 
           let n_y: u32 = rng.gen_range(0, self.height); 
           let n_x: u32 = rng.gen_range(0, self.width); 
           if * self.get_cell(n_y, n_x) != Cell::Empty {
               continue;
           } else { // is empty
               self.set_cell(n_y, n_x, Cell::Apple);
               return Some((n_y, n_x));
           }
       }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Snake_body,
    Apple,
    Empty,
}


impl Game {
    pub fn new() -> Game {
        initscr();
        noecho();
        let mut y_max: i32 = 0;
        let mut x_max: i32 = 0;
        getmaxyx(stdscr(),&mut y_max,&mut x_max);

        if y_max < 20 || x_max < 20 {
            let msg = format!("This terminal ({}x{}) is too small.
Need a bigger terminal.
Need at least ({}x{})

Press anything to exit.", y_max, x_max, LEAST_HEIGHT_FOR_GAME, LEAST_WIDTH_FOR_GAME);
            printw(msg.as_str());
            getch();
            std::process::exit(0);
        }

        let board_height = y_max - 4;
        let board_width  = x_max - 4;

        let pause_y = (board_height / 2 - 2) as i32;
        let pause_x = (board_width / 2 - 8) as i32;

        let msg_y = 0;
        let msg_x = 1;

        Game { 
            board_win: newwin(board_height, board_width, 2, 2),
            pause_win: newwin(5, 20, pause_y, pause_x),
            info_win: newwin(2 , board_width, y_max - 2, 1),
            mesg_win: newwin(2, 96, msg_y, msg_x),
            board: Board::new(board_height as u32, board_width as u32),
            refresh_time: Duration::from_millis(500),
            difficulty: 0,
        }
    }

    fn update_board_win(&self, cell: (u32, u32), ch: char) {
       mvwaddch(self.board_win, cell.0 as i32, cell.1 as i32, ch as u32);
    }

    pub fn setup(&self) {
        box_(self.board_win, '|' as u32, '-' as u32);
        keypad(self.board_win, true);
    }
    
    fn tick(snake: &mut Snake) {
        snake.keep_moving_one_step();
    }

    fn wait(&self) {
        thread::sleep(self.refresh_time);
    }

    unsafe fn toggle_pause(&self) {
        if PAUSED {
            self.unset_pause();
            PAUSED = false;
        } else {
            self.set_pause();
            PAUSED = true;
        }
    }

    fn set_pause(&self) {
        nodelay(self.board_win, false);
        box_(self.pause_win, '*' as u32, '*' as u32);
        mvwprintw(self.pause_win, 1, 2, "Paused");
        mvwprintw(self.pause_win, 2, 2, "p or space key");
        mvwprintw(self.pause_win, 3, 2, "to unpause");
        wrefresh(self.pause_win);
    }

    fn unset_pause(&self) {
        werase(self.pause_win);
        werase(self.mesg_win);
        wrefresh(self.pause_win);
        wrefresh(self.board_win);
        wrefresh(self.mesg_win);
        //wgetch(self.board_win);
        nodelay(self.board_win, true);
    }

    fn run_command(&self, command: & Command, snake: &mut Snake) -> Result<(), String>{
        match command {
            Command::Ignore => {
                Ok(())
            },
            Command::Up | Command::Down | Command::Left | Command::Right => {
                unsafe {
                    if PAUSED {
                        CHEAT_COUNT += 1;
                        mvwprintw(self.mesg_win, 0, 1, "You should not move in paused mode.");
                        please_dont_cheat(& self.mesg_win);
                        wrefresh(self.mesg_win);
                    }
                }
                snake.change_direction(
                    & command_to_direction(& command).unwrap());
                Ok(())
            },
            Command::Quit => {
                self.end();
                std::process::exit(0);
            },
            Command::Pause => {
                unsafe {
                    self.toggle_pause();
                }
                Ok(())
            },
            _ => Err(format!("Command is: {:?}", command))
        } 
    }

    pub fn play(&mut self) {
        let mut snake = Snake::new(& self.board_win, self.board.height -1 , self.board.width -1);
        let mut counter=100;
        nodelay(self.board_win, true);
        loop {
            snake.show_snake_head();
            self.run_command(& get_command(& self.board_win), &mut snake)
                .unwrap_or_else(|err| {
                    mvwprintw(self.mesg_win, 0, 0, format!("Something wrong: {}", err).as_str()); 
                    wrefresh(self.mesg_win);
                });
            Game::tick(&mut snake);
            self.wait();

            // temp time info
            self.refresh_time = Duration::from_millis(100);
            mvwprintw(self.info_win, 0, 1, format!("Counter is: {}; Duration is {:?}", counter,  self.refresh_time).as_str());
            wrefresh(self.info_win);

            counter += 1;
            if counter % 3 == 0 {
                //self.update_board_win(self.board.generate_apple().unwrap(), 'a');
            }
            if counter == 1 { break; }
        }

        wrefresh(self.board_win);
        wgetch(self.board_win);
    }

    pub fn end(&self) {
        endwin();
    }
}

fn get_command(win: &WINDOW) -> Command {
    let key = wgetch(* win);
    if DIRECTION_KEYS[0].contains(& key) {
        Command::Up
    } else if DIRECTION_KEYS[1].contains(& key) {
        Command::Down
    } else if DIRECTION_KEYS[2].contains(& key) {
        Command::Left
    } else if DIRECTION_KEYS[3].contains(& key) {
        Command::Right
    } else if key == 'q' as i32 {
        Command::Quit
    } else if key == 'p' as i32 || key == ' ' as i32 {
        Command::Pause
    } else if key == '\n' as i32 {
        Command::Enter
    } else if key == KEY_F1 {
        Command::Help
    } else if key == 'c' as i32 {
        Command::Config
    } else if key == -1 {
        Command::Ignore
    } else {
        Command::Unknown
    }
}

unsafe fn please_dont_cheat(win: &WINDOW) {
    if CHEAT_COUNT >= 50 && CHEAT_COUNT < CHEAT_WARNING_NUM {
        mvwprintw(*win, 1, 1, " YOU CHEATER !!! CHEATER !!!");
    } else if CHEAT_COUNT >= CHEAT_WARNING_NUM && CHEAT_COUNT < CHEAT_WARNING2_NUM {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. I DON'T CARE ANYTHING MORE", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT >= CHEAT_WARNING2_NUM && CHEAT_COUNT < CHEAT_WARNING3_NUM {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. WHEN WILL YOU STOP CHEATING ??? CHEATER ! ! ! ! !", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT == CHEAT_WARNING3_NUM {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. Do you wanna play my another game tictactoe-rust", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT == CHEAT_WARNING3_NUM  +  1 {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. HERE: https://github.com/alexlai97/tictactoe-rust", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT > CHEAT_WARNING3_NUM + 1  && CHEAT_COUNT < CHEAT_MAX_NUM {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. WHEN WILL YOU STOP CHEATING ??? CHEATER ! ! ! ! !", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT == CHEAT_MAX_NUM {
        mvwprintw(*win, 1, 1, format!("CHEATING ATTEMPTS: {}. ONE MORE TIME, I WILL KILL THE GAME (with memory leak to your terminal) ", CHEAT_COUNT).as_str());
    } else if CHEAT_COUNT == CHEAT_MAX_NUM + 1 {
        std::process::exit(1);
    } else {
        mvwprintw(*win, 1, 1, "But I don't see anything!");
    }
}

fn command_to_direction(c: &Command) -> Option<Direction> {
    match c {
        Command::Up => Some(Direction::Up),
        Command::Down => Some(Direction::Down),
        Command::Left => Some(Direction::Left),
        Command::Right => Some(Direction::Right),
        _ => None,
    }
}

#[derive(PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum Command {
    Quit,
    Pause,
    Enter,
    Help,
    Config,
    Up, 
    Down,
    Left,
    Right,
    Ignore,
    Unknown,
}

static DIRECTION_KEYS: &'static[[i32;3]; 4] = &[
    [ KEY_UP    , ('w' as i32) , ('k' as i32) ],  
    [ KEY_DOWN  , ('s' as i32) , ('j' as i32) ],
    [ KEY_LEFT  , ('a' as i32) , ('h' as i32) ],
    [ KEY_RIGHT , ('d' as i32) , ('l' as i32) ],
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_test() {
       let mut board: Board = Board::new(10, 10); 
        board.set_cell(3,4, Cell::Apple);
        assert!(board.grid[3][4] == Cell::Apple && 
                board.grid[4][3] != Cell::Apple && 
                board.grid[3][3] != Cell::Apple && 
                board.grid[2][4] != Cell::Apple);
    }
} /* tests */
