use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::ops::Add;

fn main() {
    let text = fs::read_to_string("input.txt").unwrap();
    let wires = parse_wires(text);
    println!(
        "Closest intersection: {:?}",
        find_closest_intersection(&wires[0], &wires[1])
    );

    println!(
        "Intersection with minimal steps requires {} steps",
        find_minimal_step_intersection(&wires[0], &wires[1]).unwrap()
    );
}

fn parse_wires(text: String) -> Vec<Wire> {
    // Read moves
    let moves_of_wires: Vec<Vec<Move>> = text
        .trim()
        .split('\n') // First split -- on two wires
        .map(|wire_text| {
            wire_text
                .trim()
                .split(',')
                .map(|string| string.into())
                .collect()
        })
        .collect();

    let origin = Point { x: 0, y: 0 };

    // Calculate every point of each wire
    let mut wires: Vec<Wire> = moves_of_wires
        .iter()
        .map(|wire_moves| {
            // Every wire starts at origin
            let mut current = origin;
            let mut before = origin;

            wire_moves
                .iter()
                .map(|&move_| {
                    let mut points = vec![];
                    // Calculate the end of a segment
                    before = current;
                    current = current + move_;

                    // Store points of this segment
                    points.extend(all_points_between(before, current));
                    points.push(current);

                    // Return all points of this siegment
                    points
                })
                // Collect points of segments in a flat vector of all points of a wire
                .flatten()
                .collect()
        })
        .collect();

    // Add origin to all of the wires
    for wire in &mut wires {
        wire.insert(0, origin);
    }

    wires
}

fn all_points_between(first: Point, second: Point) -> Vec<Point> {
    let mut points = Vec::new();

    // Which way to go along x and along y
    let step = Point {
        x: match second.x.cmp(&first.x) {
            Ordering::Greater => 1,
            Ordering::Equal => 0,
            Ordering::Less => -1,
        },
        y: match second.y.cmp(&first.y) {
            Ordering::Greater => 1,
            Ordering::Equal => 0,
            Ordering::Less => -1,
        },
    };

    assert!(
        step.x == 0 || step.y == 0,
        "Two points should lie on the same axis: either X or Y coordinates are equal"
    );

    if (second.x - first.x).abs() == 1 || (second.y - first.y).abs() == 1 {
        return vec![];
    }

    let mut current = first;

    loop {
        current = Point {
            x: current.x + step.x,
            y: current.y + step.y,
        };
        if current == second {
            break;
        }
        points.push(current);
    }

    points
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(chr: char) -> Self {
        match chr {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Move {
    direction: Direction,
    distance: u32,
}

impl From<&str> for Move {
    fn from(string: &str) -> Self {
        Move {
            direction: string.chars().next().unwrap().into(),
            distance: string.get(1..).unwrap().parse().unwrap(),
        }
    }
}

type Wire = Vec<Point>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn distance_from_origin(&self) -> u32 {
        (self.x.abs() + self.y.abs()) as u32
    }
}

impl Ord for Point {
    /// Compare Points by Manhattan distance between a point and an origin (Point {x: 0, y: 0}).
    /// Manhattan distance is sum of absolute values of every coordinate of the Point.
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance_from_origin()
            .cmp(&other.distance_from_origin())
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Add<Move> for Point {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, move_: Move) -> Self::Output {
        match move_.direction {
            Direction::Left => Point {
                x: self.x - move_.distance as i32,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + move_.distance as i32,
                y: self.y,
            },
            Direction::Up => Point {
                x: self.x,
                y: self.y + move_.distance as i32,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y - move_.distance as i32,
            },
        }
    }
}

/// Find intersections of wires. Intersection is guaranteed to be included as a Point in a wire
/// vec as all lines are aligned along axis and all moves' distances are integer.
fn find_intersections(left: &[Point], right: &[Point]) -> HashSet<Point> {
    let left_set: HashSet<Point> = left.iter().cloned().collect();
    let right_set: HashSet<Point> = right.iter().cloned().collect();
    let mut intersections = left_set
        .intersection(&right_set)
        .cloned()
        .collect::<HashSet<_>>();
    intersections.remove(&Point { x: 0, y: 0 });
    intersections
}

/// Find closest to origin intersection of two wires.
fn find_closest_intersection(left: &[Point], right: &[Point]) -> Option<Point> {
    find_intersections(left, right).iter().min().cloned()
}

/// Find intersection of wires that produces minimal delay. Sum of lengths of wires till this
/// intersection should be minimal.
fn find_minimal_step_intersection(left: &[Point], right: &[Point]) -> Option<u32> {
    find_intersections(left, right)
        .iter()
        .map(|&point| length_to_point_in_wire(left, point) + length_to_point_in_wire(right, point))
        .min()
}

/// Compute length of a wire segment from origin to this point.
fn length_to_point_in_wire(wire: &[Point], point: Point) -> u32 {
    wire.iter().position(|&p| p == point).unwrap() as u32
}

#[test]
fn test_find_closest_intersection() {
    let cases = [
        (
            [
                [Point { x: 0, y: 6 }, Point { x: 5, y: 0 }],
                [Point { x: 5, y: 0 }, Point { x: 0, y: 6 }],
            ],
            Some(Point { x: 5, y: 0 }),
        ),
        (
            [
                [Point { x: 0, y: 6 }, Point { x: 5, y: 0 }],
                [Point { x: 0, y: 6 }, Point { x: 5, y: 0 }],
            ],
            Some(Point { x: 5, y: 0 }),
        ),
    ];

    for case in &cases {
        assert_eq!(
            find_closest_intersection(&case.0[0], &case.0[1]),
            Some(Point { x: 5, y: 0 })
        )
    }
}

#[test]
fn test_point_ordering() {
    let point1 = Point { x: 0, y: 6 };
    let point2 = Point { x: 5, y: 0 };
    assert_eq!(point1 > point2, true);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: -3, y: 6 };
    let point2 = Point { x: 5, y: 0 };
    assert_eq!(point1 > point2, true);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: 0, y: 6 };
    let point2 = Point { x: -5, y: 0 };
    assert_eq!(point1 > point2, true);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: 0, y: -6 };
    let point2 = Point { x: 5, y: 0 };
    assert_eq!(point1 > point2, true);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: 5, y: 5 };
    let point2 = Point { x: 5, y: 5 };
    assert_eq!(point1 > point2, false);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: -10, y: -10 };
    let point2 = Point { x: 5, y: 0 };
    assert_eq!(point1 > point2, true);
    assert_eq!(point1 < point2, false);

    let point1 = Point { x: 0, y: 3 };
    let point2 = Point { x: 5, y: 2 };
    assert_eq!(point1 > point2, false);
    assert_eq!(point1 < point2, true);
}

#[test]
fn test_wire_parsing() {
    let text = "R2,U2,L3,D1\nL1,U2,R3".to_owned();
    let wires = parse_wires(text);
    print_wires(&wires);
    assert_eq!(
        wires,
        [
            vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 2, y: 0 },
                Point { x: 2, y: 1 },
                Point { x: 2, y: 2 },
                Point { x: 1, y: 2 },
                Point { x: 0, y: 2 },
                Point { x: -1, y: 2 },
                Point { x: -1, y: 1 },
            ],
            vec![
                Point { x: 0, y: 0 },
                Point { x: -1, y: 0 },
                Point { x: -1, y: 1 },
                Point { x: -1, y: 2 },
                Point { x: 0, y: 2 },
                Point { x: 1, y: 2 },
                Point { x: 2, y: 2 },
            ]
        ]
    )
}

#[test]
fn test_wire_crossing_distance() {
    let cases = vec![
        (
            vec![
                vec![Point { x: 0, y: 5 }, Point { x: 0, y: 6 }],
                vec![Point { x: 0, y: 6 }, Point { x: 1, y: 6 }],
            ],
            6,
        ),
        (
            parse_wires(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83".to_owned(),
            ),
            159,
        ),
        (
            parse_wires(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
                    .to_owned(),
            ),
            135,
        ),
    ];

    for case in cases {
        let wires = case.0;
        let distance = case.1;
        print_wires(&wires);
        let intersection = find_closest_intersection(&wires[0], &wires[1]);
        assert_eq!(intersection.unwrap().distance_from_origin(), distance);
    }

    assert_eq!(
        find_closest_intersection(
            &[Point { x: 0, y: 0 }, Point { x: 0, y: 1 }],
            &[Point { x: 1, y: 2 }, Point { x: 2, y: 2 }]
        ),
        None
    );
}

#[test]
fn test_all_points_between() {
    let cases = vec![
        ((Point { x: 0, y: 0 }, Point { x: 1, y: 0 }), vec![]),
        (
            (Point { x: 0, y: 0 }, Point { x: 2, y: 0 }),
            vec![Point { x: 1, y: 0 }],
        ),
        (
            (Point { x: -1, y: 0 }, Point { x: 1, y: 0 }),
            vec![Point { x: 0, y: 0 }],
        ),
        ((Point { x: -1, y: 0 }, Point { x: -2, y: 0 }), vec![]),
        (
            (Point { x: -1, y: 0 }, Point { x: -3, y: 0 }),
            vec![Point { x: -2, y: 0 }],
        ),
        ((Point { x: 1, y: 1 }, Point { x: 1, y: 1 }), vec![]),
        (
            (Point { x: 5, y: 6 }, Point { x: 5, y: 10 }),
            vec![
                Point { x: 5, y: 7 },
                Point { x: 5, y: 8 },
                Point { x: 5, y: 9 },
            ],
        ),
    ];

    for case in cases {
        assert_eq!(all_points_between(case.0.0, case.0.1), case.1, "\nfailed case {:?}", case);
    }
}

#[test]
#[should_panic]
fn test_all_points_between_panics_when_points_are_not_on_the_same_axis() {
    all_points_between(Point { x: 0, y: 1 }, Point { x: 1, y: 2 });
}

fn print_wires(wires: &[Vec<Point>]) {
    for (p1, p2) in wires[0].iter().zip(wires[1].iter()) {
        println!("{:5?}       {:5?}", p1, p2);
    }
}
