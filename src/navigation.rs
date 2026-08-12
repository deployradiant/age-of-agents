use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

use crate::game::CellCoordinate;

pub fn find_path(
    columns: u16,
    rows: u16,
    start: CellCoordinate,
    goal: CellCoordinate,
    blocked: &BTreeSet<CellCoordinate>,
) -> Option<Vec<CellCoordinate>> {
    if start.column >= columns
        || start.row >= rows
        || goal.column >= columns
        || goal.row >= rows
        || (start != goal && blocked.contains(&goal))
    {
        return None;
    }
    if start == goal {
        return Some(Vec::new());
    }

    let cell_count = usize::from(columns) * usize::from(rows);
    let mut best_cost = vec![u32::MAX; cell_count];
    let mut came_from = vec![None; cell_count];
    let mut open = BinaryHeap::new();
    best_cost[index(columns, start)] = 0;
    open.push(Reverse((heuristic(start, goal), 0_u32, start)));

    while let Some(Reverse((_, cost, current))) = open.pop() {
        if cost != best_cost[index(columns, current)] {
            continue;
        }
        if current == goal {
            return Some(reconstruct_path(columns, &came_from, start, goal));
        }

        for neighbor in neighbors(current, columns, rows) {
            if blocked.contains(&neighbor) {
                continue;
            }
            let candidate = cost + 1;
            let neighbor_index = index(columns, neighbor);
            if candidate < best_cost[neighbor_index] {
                best_cost[neighbor_index] = candidate;
                came_from[neighbor_index] = Some(current);
                open.push(Reverse((
                    candidate + heuristic(neighbor, goal),
                    candidate,
                    neighbor,
                )));
            }
        }
    }
    None
}

pub fn neighbors(
    coordinate: CellCoordinate,
    columns: u16,
    rows: u16,
) -> impl Iterator<Item = CellCoordinate> {
    let mut result = Vec::with_capacity(4);
    if coordinate.row > 0 {
        result.push(CellCoordinate {
            column: coordinate.column,
            row: coordinate.row - 1,
        });
    }
    if coordinate.column > 0 {
        result.push(CellCoordinate {
            column: coordinate.column - 1,
            row: coordinate.row,
        });
    }
    if coordinate.column + 1 < columns {
        result.push(CellCoordinate {
            column: coordinate.column + 1,
            row: coordinate.row,
        });
    }
    if coordinate.row + 1 < rows {
        result.push(CellCoordinate {
            column: coordinate.column,
            row: coordinate.row + 1,
        });
    }
    result.sort_unstable();
    result.into_iter()
}

fn index(columns: u16, coordinate: CellCoordinate) -> usize {
    usize::from(coordinate.row) * usize::from(columns) + usize::from(coordinate.column)
}

fn heuristic(from: CellCoordinate, to: CellCoordinate) -> u32 {
    u32::from(from.column.abs_diff(to.column) + from.row.abs_diff(to.row))
}

fn reconstruct_path(
    columns: u16,
    came_from: &[Option<CellCoordinate>],
    start: CellCoordinate,
    goal: CellCoordinate,
) -> Vec<CellCoordinate> {
    let mut path = vec![goal];
    let mut current = goal;
    while current != start {
        current = came_from[index(columns, current)].expect("reached nodes have predecessors");
        if current != start {
            path.push(current);
        }
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_is_deterministic_and_avoids_blocked_cells() {
        let start = CellCoordinate { column: 0, row: 0 };
        let goal = CellCoordinate { column: 2, row: 0 };
        let blocked = BTreeSet::from([CellCoordinate { column: 1, row: 0 }]);
        let expected = vec![
            CellCoordinate { column: 0, row: 1 },
            CellCoordinate { column: 1, row: 1 },
            CellCoordinate { column: 2, row: 1 },
            goal,
        ];
        for _ in 0..20 {
            assert_eq!(
                find_path(3, 2, start, goal, &blocked),
                Some(expected.clone())
            );
        }
    }

    #[test]
    fn enclosed_goal_is_unreachable() {
        let blocked = BTreeSet::from([
            CellCoordinate { column: 1, row: 0 },
            CellCoordinate { column: 0, row: 1 },
        ]);
        assert_eq!(
            find_path(
                3,
                3,
                CellCoordinate { column: 0, row: 0 },
                CellCoordinate { column: 2, row: 2 },
                &blocked,
            ),
            None
        );
    }
}
