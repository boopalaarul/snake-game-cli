//standard methods, can import into any project
//use std::time::Duration;
//use std::thread::sleep;
use std::io;
use std::io::Write; //provides stdout flush
use std::collections::VecDeque;
use std::collections::BTreeSet;

use rand::Rng;

const SIZE : usize = 10; //equality checks demand same type of integer, can cast

//could build an enum out of this, but will end up using same hardcoded values anyways 
const PIXEL_DEFAULT : char = '.';
const PIXEL_TREAT : char = '*';
const PIXEL_SNAKE : char = 'X';

const TREAT_CHANCE : f64 = 0.1;

enum Direction {
    Up,
    Down,
    Left,
    Right
}
const GRID_BLANK : [[char; SIZE]; SIZE] = [[PIXEL_DEFAULT; SIZE]; SIZE];

//main function not allowed to be async, async methods in body resolve whenever they resolve
//(can maybe put main function's thread to sleep and hope resolution happens before then)
fn main() {
    //initialize array
    let mut game_grid : [[char; SIZE]; SIZE] = GRID_BLANK;

    //initialize a set of treats: set should persist between loops, which means
    //add_treats() should take a reference to set as its parameter
    //coordinates are (row, col), not (x, y)
    let mut treats : BTreeSet<(usize, usize)> = BTreeSet::new();
    treats.insert((3, 4));
    
    //initialize snake
    let start : (usize, usize) = (5,5);
    let mut snake : VecDeque<(usize, usize)> = VecDeque::from([start, (start.0 + 1, start.1)]);

    //initialize snake direction
    let mut direction : Direction = Direction::Up;

    //event loop
    loop {
        //1) add treats
        add_treats(&mut game_grid, &treats);
        //2) add snake
        add_snake(&mut game_grid, &snake);
        //3) draw & begin accepting key input-- state machine where each state can only lead
        //to one of three other states (snake in DIR_UP can only switch to left, right, or up)
        //or, why not have only two keys: turn left and turn right?
        draw(&game_grid);
        direction = update_direction(direction);
        //4) close key input and calculate next_head
        let next_head : (isize, isize) = next_head(&snake, &direction);
        //5) check if game over
        if is_game_over(next_head, &snake) {
            break;
        }

        //6) if not game over, start preparing for next frame: update_snake
        update_snake_treats(&mut snake, &mut treats, next_head);
        
        //7) if not game over, 10% chance per frame of new treat in random location IF location 
        //is not occupied by snake (can use binary search provided by VecDeque...?)
        generate_treats(&mut treats, &snake);

        //8) start at 1) with a blank grid
        game_grid = GRID_BLANK;

        //ANSI escape string, moves up SIZE + 3 rows: 
        //extra rows: one for input prompt, one for input (input + newline) 
        print!("\x1b[{}A", SIZE + 2);
        std::io::stdout().flush().expect("Flush failed");

    }; //loop shouldn't have any ultimate return value
}

/* Iterate through BTreeSet items and for each, switch pixel on game_grid */
fn add_treats(game_grid: &mut [[char; SIZE]; SIZE], treats: &BTreeSet<(usize, usize)>) {
    for coords in treats.iter() {
        game_grid[coords.0][coords.1] = PIXEL_TREAT;
    }
}

/* Switches pixels on grid corresponding to body of the snake. */
fn add_snake(game_grid: &mut [[char; SIZE]; SIZE], snake: &VecDeque<(usize, usize)>) {
    for coords in snake.iter() {
        game_grid[coords.0][coords.1] = PIXEL_SNAKE;
    }
}

/* Reads in user input, changes direction.*/
fn update_direction(direction: Direction) -> Direction {
    let mut input = String::new();
    match direction {
        Direction::Up | Direction::Down => {
            println!("Enter A or D to change direction.");
            io::stdin().read_line(&mut input).expect("Failed input");
            match input.as_str() {
                "A\n" => Direction::Left,
                "D\n" => Direction::Right,
                _ => direction
            }
        },
        Direction::Left | Direction::Right => {
            println!("Enter W or S to change direction.");
            io::stdin().read_line(&mut input).expect("Failed input");
            match input.as_str() {
                "W\n" => Direction::Up,
                "S\n" => Direction::Down,
                _ => direction
            }
        }
    }
}

fn next_head(snake: &VecDeque<(usize, usize)>, direction: &Direction) -> (isize, isize) {
    //snake.front returns an Option, which is either Some(T)or None; if it's Some then
    //the value used to generate the Option can be unpacked and, if reference, dereferenced
    //Options unwrapped with match, Result unwrapped with expect
    match snake.front() {
        //final expression and return value of function
        //converting to isize allows return values to be negative
        //you can convert a `usize` to an `isize` and panic if the converted value doesn't fit: `(`, `).try_into().unwrap()`
        Some(&coords) => match direction {
            Direction::Up => (isize::try_from(coords.0).expect("u->i failed") - 1,
                                isize::try_from(coords.1).expect("u->i failed")),
            Direction::Down => (isize::try_from(coords.0).expect("u->i failed") + 1,
                                isize::try_from(coords.1).expect("u->i failed")),
            Direction::Left => (isize::try_from(coords.0).expect("u->i failed"),
                                isize::try_from(coords.1).expect("u->i failed") - 1),
            Direction::Right => (isize::try_from(coords.0).expect("u->i failed"),
                                isize::try_from(coords.1).expect("u->i failed") + 1),
            //_ => default case
        },
        None => panic!("Snake deque is empty"),
    }
}

fn is_game_over(next_head: (isize, isize), snake: &VecDeque<(usize, usize)>) -> bool {
    //if next_head is past any of the walls, return true
    let size = isize::try_from(SIZE).expect("u->i failed");
    next_head.0 < 0 || 
    next_head.0 > size - 1 || 
    next_head.1 < 0 || 
    next_head.1 > size - 1 ||
    //if next head overlaps a position in snake UNLESS that position is snake.back()
    {
        //can now be sure that next_head.0 and .1 are within [0, SIZE)
        let next_head = (usize::try_from(next_head.0).expect("panic"),
                        usize::try_from(next_head.1).expect("panic"));
        snake.contains(&next_head) &&
        match snake.back() {
            Some(&coords) => coords != next_head,
            None => panic!("Snake deque is empty")
        }
    }
}

fn update_snake_treats(snake: &mut VecDeque<(usize, usize)>, treats: &mut BTreeSet<(usize, usize)>, next_head: (isize, isize)) {
    let next_head = (usize::try_from(next_head.0).expect("i->u conversion fail"),
                    usize::try_from(next_head.1).expect("i->u conversion fail"));
    //push next head onto front of snake
    snake.push_front(next_head);
    //if next head overlaps a treat, don't pop back (allow snake to grow) 
    //and remove treat from set
    if treats.contains(&next_head) {
        treats.remove(&next_head);
    } else {
        snake.pop_back();
    }
}

fn generate_treats(treats: &mut BTreeSet<(usize, usize)>, snake: &VecDeque<(usize, usize)>) {
    let to_generate : f64 = f64::try_from(rand::thread_rng().gen_range(0..99)).expect("int->f32 failed");
    
    //generation has a (TREAT_CHANCE)% chance of happening per frame
    if to_generate  < 100.0 * TREAT_CHANCE {
        
        //if the randomly generated grid point overlaps with snake, pick another point
        //problem: if snake gets big enough, more and more time will be spent in this loop
        let new_treat = loop {
            let random_row : usize = rand::thread_rng().gen_range(0..SIZE);
            let random_col : usize = rand::thread_rng().gen_range(0..SIZE);
            if !snake.contains(&(random_row, random_col)) {
                break (random_row, random_col)
            }
        };
        
        treats.insert(new_treat);
    }
}

//^C may disfigure final output... might not be any way to prevent it from showing up, but 
//it would be nice to handle the SIGINT more gracefully (finish the current loop, break, etc)
//the terminal cursor also gets in the way of the animation...

/* No more changing the game grid, we're just going to draw it */
fn draw(game_grid: &[[char; SIZE]; SIZE]) {
    for row in game_grid {
        for pixel in row {
            print!("{pixel} "); //seems to work without flush for now, maybe only bc main() exits
            std::io::stdout().flush().expect("Flush failed");    
        }
        print!("\n");
        std::io::stdout().flush().expect("Flush failed");
    }
}
