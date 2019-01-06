use ncurses::*;
use super::game::Direction;

pub struct Snake<'a> {
    head_y: u32,
    head_x: u32,
    body_reversed: Vec<(u32, u32)>, // tail is index 0, head is len - 1
    max_y: u32,
    max_x: u32,
    curwin: &'a WINDOW,
    direction: Direction,
}



impl <'a> Snake<'a> {
    pub fn new(curwin: &'a WINDOW, max_y: u32, max_x: u32) -> Snake {
        let head_y = 10; // FIXME: random
        let head_x = 10;
        let mut body_reversed = Vec::new();
        body_reversed.push((head_y, head_x));
        Snake { 
            head_y,
            head_x,
            body_reversed, 
            max_y,
            max_x,
            curwin,
            direction: Direction::Right, // FIXME: random
        }
    }

    pub fn show_snake_head(&self) {
        mvwaddch(* self.curwin, self.head_y as i32, self.head_x as i32, '@' as u32);
    }

    pub fn get_direction(&self) -> & Direction {
        & self.direction
    }


    fn move_up(&mut self) {
        mvwaddch(* self.curwin, self.head_y as i32, self.head_x as i32, '.' as u32);
        if self.head_y <= 1 {
            self.head_y = self.max_y - 1;
        } else {
            self.head_y -= 1;
        }
    }

    fn move_down(&mut self) {
        mvwaddch(* self.curwin, self.head_y as i32, self.head_x as i32, '.' as u32);
        if self.head_y >= self.max_y - 1 {
            self.head_y = 1;
        } else {
            self.head_y += 1;
        } 
    }

    fn move_left(&mut self) {
        mvwaddch(* self.curwin, self.head_y as i32, self.head_x as i32, '.' as u32);
        if self.head_x <= 1 {
            self.head_x = self.max_x - 1;
        } else if self.head_x == 2{
            self.head_x = 1;
        } else {
            self.head_x -= 2;
        }
    }

    fn move_right(&mut self) {
        mvwaddch(* self.curwin, self.head_y as i32, self.head_x as i32, '.' as u32);
        if self.head_x >= self.max_x - 1{
            self.head_x = 1;
        } else if self.head_x == self.max_x - 2 {
            self.head_x = self.max_x - 1;
        } else {
            self.head_x += 2;
        } 
    }

    pub fn keep_moving_one_step(&mut self) {
        match self.direction {
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
        }
    }

    pub fn change_direction(&mut self, d: &Direction) {
        if ! conflict_route(&self.direction, d) {
            self.direction = d.clone();
        }
    }
}

fn conflict_route(d1: &Direction, d2: &Direction) -> bool {
    opposite_direction(d1, d2) || same_direction(d1, d2)    
}

fn opposite_direction(d1: &Direction, d2: &Direction) -> bool {
   match (d1, d2) {
        (Direction::Up, Direction::Down) => true,
        (Direction::Down, Direction::Up) => true,
        (Direction::Left, Direction::Right) => true,
        (Direction::Right, Direction::Left) => true,
        _ => false,
   } 
}

fn same_direction(d1: &Direction, d2: &Direction) -> bool {
   d1 == d2 
}
