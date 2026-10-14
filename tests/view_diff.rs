use v_ide_view::{Ansi, Color, Grid, Op, Style, diff};

fn red() -> Style {
    Style {
        fg: Color::Ansi(Ansi::Red),
        ..Style::default()
    }
}

fn text(s: &str) -> Op {
    Op::Text(s.to_string())
}

#[test]
fn identical_grids_produce_nothing() {
    let mut grid = Grid::new(4, 2);
    grid.put_str(0, 0, "ab", red(), 10);
    assert_eq!(diff(&grid, &grid.clone()), vec![]);
}

#[test]
fn single_run_moves_styles_and_writes_once() {
    let prev = Grid::new(5, 2);
    let mut next = Grid::new(5, 2);
    next.put_str(2, 1, "ab", Style::default(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 2, y: 1 },
            Op::Style(Style::default()),
            text("ab")
        ]
    );
}

#[test]
fn gap_needs_move_but_same_style_is_not_repeated() {
    let prev = Grid::new(5, 1);
    let mut next = Grid::new(5, 1);
    next.put_str(0, 0, "a", red(), 10);
    next.put_str(3, 0, "b", red(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 0, y: 0 },
            Op::Style(red()),
            text("a"),
            Op::Move { x: 3, y: 0 },
            text("b"),
        ]
    );
}

#[test]
fn style_change_inside_run_emits_style_without_move() {
    let prev = Grid::new(3, 1);
    let mut next = Grid::new(3, 1);
    next.put_str(0, 0, "a", red(), 10);
    next.put_str(1, 0, "b", Style::default(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 0, y: 0 },
            Op::Style(red()),
            text("a"),
            Op::Style(Style::default()),
            text("b"),
        ]
    );
}

#[test]
fn runs_do_not_wrap_across_rows() {
    let prev = Grid::new(2, 2);
    let mut next = Grid::new(2, 2);
    next.put_str(1, 0, "a", Style::default(), 10);
    next.put_str(0, 1, "b", Style::default(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 1, y: 0 },
            Op::Style(Style::default()),
            text("a"),
            Op::Move { x: 0, y: 1 },
            text("b"),
        ]
    );
}

#[test]
fn wide_char_skips_its_continuation_and_keeps_the_run() {
    let prev = Grid::new(4, 1);
    let mut next = Grid::new(4, 1);
    next.put_str(0, 0, "中b", Style::default(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 0, y: 0 },
            Op::Style(Style::default()),
            text("中b")
        ]
    );
}

#[test]
fn erased_cell_is_written_as_blank() {
    let mut prev = Grid::new(2, 1);
    prev.put_str(0, 0, "a", red(), 10);
    let next = Grid::new(2, 1);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Move { x: 0, y: 0 },
            Op::Style(Style::default()),
            text(" ")
        ]
    );
}

#[test]
fn size_change_clears_and_writes_non_blank_cells() {
    let mut prev = Grid::new(3, 1);
    prev.put_str(0, 0, "abc", red(), 10);
    let mut next = Grid::new(4, 1);
    next.put_str(1, 0, "a", Style::default(), 10);
    assert_eq!(
        diff(&prev, &next),
        vec![
            Op::Clear,
            Op::Move { x: 1, y: 0 },
            Op::Style(Style::default()),
            text("a"),
        ]
    );
}
