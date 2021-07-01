use std::fmt;
use std::io::{self, BufRead};
use std::str::FromStr;

// As an example 3,100,2#F-12:6-100,F-13:12-20,E-9:5-100,E-9:12-90#123 would mean 3,100,2 is Game Data, F-12:6-100,F-13:12-20,E-9:5-100,E-9:12-90 is Map State, and 123 is User Data.
// As an example 3,100,2 suggests the game is at turn 3 of 100, and we are player 2 in the game.
//  As an example F-12:6-100,F-13:12-20,E-9:5-100,E-9:12-90 would be a valid input that states the current arena state is:

//    Friendly at x12 y6 with health 100
//    Friendly at x13 y12 with health 20
//    Enemy at x9 y5 with health 100
//    Enemy at x9 y12 with health 90

//     Coordinates - The first sub element will be a set of coordinates for the friendly robot, which this instruction relates to. The coordinates are the placement of the bot at the start of the turn before any actions during the turn, that is exactly as provided in the stdin input.
//     Action – The second element will define the action that is to take place, with the possible actions defined below, and detailed further within the game rules.
//         Attack - Encoded as A (Requires a direction element)
//         Move - Encoded as M (Requires a direction element)
//         Defend - Encoded as D
//         Self Destruct - Encoded as S
//     Direction – Some actions (such as move or attack) are directional, therefore where a direction is required the third element will be included to provide this detail, the optional directions are:
//         North / Up – Encoded as N or U
//         East / Right – Encoded as E or R
//         South / Down – Encoded as S or D
//         West / Left – Encoded as W or L

//  As an example 12:7-A-S,10:5-M-E,10:12-D would be a valid output that requests:

//    Friendly at x12 y7 should attack, in a south direction, therefore x12 y6
//    Friendly at x10 y5 should move, in an east direction, tehrfore x11 y5
//    Friendly at x10 y12 should defend

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

macro_rules! err {
    ($($tt:tt)*) => { Err(Error::from(format!($($tt)*))) };
}
#[derive(PartialEq, Debug)]
enum Team {
    Friendly,
    Enemy,
}

#[derive(PartialEq, Debug)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(PartialEq, Debug)]
struct Bot {
    team: Team,
    position: Position,
    health: u8,
    action: Option<Action>,
}

#[derive(PartialEq, Debug)]
struct Board(Vec<Bot>);

#[derive(PartialEq, Debug)]
struct Game {
    board: Board,
    user_data: String,
    turn: (u32, u32),
    player_num: u8,
}

// 3,100,2#F-12:6-100,F-13:12-20,E-9:5-100,E-9:12-90#123
impl FromStr for Game {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut splitter = s.split('#');
        let (game_data, map_state, user_data) =
            match (splitter.next(), splitter.next(), splitter.next()) {
                (Some(game_data), Some(map_state), Some(user_data)) => {
                    (game_data, map_state, user_data)
                }
                _ => return err!("Couldn't parse input: {}", s),
            };

        let mut gd_splitter = game_data.split(',');
        let num = match gd_splitter.next().map(|s| s.parse::<u32>()) {
            Some(r) => r?,
            _ => return err!("No numerator"),
        };
        let denom = match gd_splitter.next().map(|s| s.parse::<u32>()) {
            Some(r) => r?,
            _ => return err!("No denominator"),
        };
        let turn = (num, denom);

        let player_num = match gd_splitter.next().map(|s| s.parse::<u8>()) {
            Some(r) => r?,
            _ => return err!("No player num"),
        };

        let board = map_state.parse()?;
        let user_data = user_data.to_string();

        Ok(Game {
            board,
            user_data,
            turn,
            player_num,
        })
    }
}

impl FromStr for Board {
    type Err = Error;
    fn from_str(map_state: &str) -> Result<Self> {
        Ok(Board(
            map_state
                .split(',')
                .map(|s| s.parse::<Bot>())
                .collect::<Result<_>>()?,
        ))
    }
}

impl FromStr for Bot {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut splitter = s.split('-');
        let team = match splitter.next() {
            Some("F") => Team::Friendly,
            Some("E") => Team::Enemy,
            _ => return err!("Couldn't parse team"),
        };
        let position = match splitter.next().map(|s| s.parse::<Position>()) {
            Some(r) => r?,
            _ => return err!("No position"),
        };
        let health = match splitter.next().map(|s| s.parse::<u8>()) {
            Some(r) => r?,
            _ => return err!("No health"),
        };

        Ok(Bot {
            team,
            position,
            health,
            action: None,
        })
    }
}

impl FromStr for Position {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        // 13:12
        let mut splitter = s.split(':');
        let (x, y) = match (splitter.next(), splitter.next()) {
            (Some(x), Some(y)) => (x, y),
            _ => return err!("Missing a position"),
        };
        match (x.parse::<usize>(), y.parse::<usize>()) {
            (Ok(x), Ok(y)) => Ok(Self { x, y }),
            _ => err!("Couldn't parse as usize: {:?}", (x, y)),
        }
    }
}

#[derive(PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Direction::*;
        write!(
            f,
            "{}",
            match self {
                Up => 'U',
                Down => 'D',
                Left => 'L',
                Right => 'R',
            }
        )
    }
}

// 12:7-A-S,10:5-M-E,10:12-D#A1-B2-C2-N6
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.board, self.user_data)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .filter(|bot| bot.team == Team::Friendly)
                .map(|bot| {
                    let action_str = if let Some(action) = &bot.action {
                        action.to_string()
                    } else {
                        Action::default().to_string()
                    };
                    format!("{}-{}", bot.position, action_str,)
                })
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.x, self.y)
    }
}

// Order of operations:
// 1. Defend
// 2. Self-Descruct starts
// 3. Move. if unable to move (e.g. both move for same spot) they will remain defenceless where they currently stand
// 4. Spawn
// 5. Attack
// 6. Self-destruct triggers

#[derive(Debug, PartialEq)]
enum Action {
    Attack(Direction),

    // When contention stops a robot from moving it will maintain its current
    // position.
    Move(Direction),
    Defend,

    // All robots immediately surrounding the self-destructing robot, will
    // receive an attack hit of 6 – This is regardless of the robot owner
    // (friendly or enemy), and surrounding is all 8 boxes in a ring around the
    // robot.
    SelfDestruct,
}

impl Default for Action {
    fn default() -> Self {
        Action::Defend
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Action::*;
        match self {
            Attack(d) => write!(f, "A-{}", d),
            Move(d) => write!(f, "M-{}", d),
            Defend => write!(f, "D"),
            SelfDestruct => write!(f, "S"),
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let game: Game = line.parse()?;
    }

    println!("5:5-D,12:12-D");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        let pos: Position = "13:12".parse().unwrap();
        assert_eq!(pos, Position { x: 13, y: 12 });
    }

    #[test]
    fn test_parse_bot() {
        let friendly: Bot = "F-13:12-20".parse().unwrap();
        let enemy: Bot = "E-9:5-100".parse().unwrap();

        assert_eq!(
            friendly,
            Bot {
                team: Team::Friendly,
                position: Position { x: 13, y: 12 },
                health: 20,
                action: None,
            }
        );
        assert_eq!(
            enemy,
            Bot {
                team: Team::Enemy,
                position: Position { x: 9, y: 5 },
                health: 100,
                action: None,
            }
        );
    }

    #[test]
    fn test_parse_board() {
        let board: Board = "E-9:5-100,F-9:12-90".parse().unwrap();
        assert_eq!(
            board,
            Board(vec![
                Bot {
                    team: Team::Enemy,
                    position: Position { x: 9, y: 5 },
                    health: 100,
                    action: None,
                },
                Bot {
                    team: Team::Friendly,
                    position: Position { x: 9, y: 12 },
                    health: 90,
                    action: None,
                },
            ])
        )
    }

    #[test]
    fn test_parse_game() {
        let game: Game = "3,100,2#F-12:6-100,E-9:12-90#123".parse().unwrap();
        assert_eq!(
            game,
            Game {
                player_num: 2,
                turn: (3, 100),
                user_data: "123".to_string(),
                board: Board(vec![
                    Bot {
                        team: Team::Friendly,
                        position: Position { x: 12, y: 6 },
                        health: 100,
                        action: None,
                    },
                    Bot {
                        team: Team::Enemy,
                        position: Position { x: 9, y: 12 },
                        health: 90,
                        action: None,
                    },
                ])
            }
        )
    }

    #[test]
    fn test_display_action() {
        let action = Action::Move(Direction::Up);
        assert_eq!(action.to_string(), String::from("M-U"));
        let action = Action::Attack(Direction::Left);
        assert_eq!(action.to_string(), String::from("A-L"));
        let action = Action::SelfDestruct;
        assert_eq!(action.to_string(), String::from("S"));
    }

    #[test]
    fn test_display_board() {
        let board = Board(vec![
            Bot {
                team: Team::Friendly,
                position: Position { x: 12, y: 6 },
                health: 100,
                action: Some(Action::Attack(Direction::Up)),
            },
            Bot {
                team: Team::Enemy,
                position: Position { x: 10, y: 12 },
                health: 90,
                action: None,
            },
            Bot {
                team: Team::Friendly,
                position: Position { x: 9, y: 12 },
                health: 90,
                action: None,
            },
        ]);
        assert_eq!(board.to_string(), "12:6-A-U,9:12-D")
    }

    #[test]
    fn test_display_game() {
        let game = Game {
            board: Board(vec![
                Bot {
                    team: Team::Friendly,
                    position: Position { x: 12, y: 6 },
                    health: 100,
                    action: Some(Action::Attack(Direction::Up)),
                },
                Bot {
                    team: Team::Enemy,
                    position: Position { x: 10, y: 12 },
                    health: 90,
                    action: None,
                },
                Bot {
                    team: Team::Friendly,
                    position: Position { x: 9, y: 12 },
                    health: 90,
                    action: None,
                },
            ]),
            user_data: "42".to_string(),
            turn: (3, 100),
            player_num: 2,
        };
        assert_eq!(game.to_string(), "12:6-A-U,9:12-D#42")
    }
}
