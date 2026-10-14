use v_ide_view::{Ansi, Cell, Color, Grid, Rect, Style, Symbol};

fn red() -> Style {
    Style {
        fg: Color::Ansi(Ansi::Red),
        ..Style::default()
    }
}

#[test]
fn new_grid_is_blank() {
    let grid = Grid::new(3, 2);
    assert_eq!((grid.w, grid.h), (3, 2));
    assert_eq!(
        grid.area(),
        Rect {
            x: 0,
            y: 0,
            w: 3,
            h: 2
        }
    );
    assert_eq!(grid.row_text(1), "   ");
    assert_eq!(*grid.cell(2, 1), Cell::default());
}

#[test]
fn put_str_writes_styled_cells_and_returns_width() {
    let mut grid = Grid::new(6, 1);
    assert_eq!(grid.put_str(1, 0, "abc", red(), 10), 3);
    assert_eq!(grid.row_text(0), " abc  ");
    assert_eq!(grid.cell(2, 0).symbol.as_str(), "b");
    assert_eq!(grid.cell(2, 0).style, red());
    assert_eq!(grid.cell(4, 0).style, Style::default());
}

#[test]
fn put_str_clips_by_max_width_and_grid_edge() {
    let mut grid = Grid::new(5, 1);
    assert_eq!(grid.put_str(0, 0, "abcdef", Style::default(), 2), 2);
    assert_eq!(grid.row_text(0), "ab   ");
    assert_eq!(grid.put_str(3, 0, "xyz", Style::default(), 10), 2);
    assert_eq!(grid.row_text(0), "ab xy");
    assert_eq!(grid.put_str(0, 5, "q", Style::default(), 10), 0);
}

#[test]
fn wide_char_takes_two_cells() {
    let mut grid = Grid::new(5, 1);
    assert_eq!(grid.put_str(0, 0, "a中b", Style::default(), 10), 4);
    assert_eq!(grid.cell(1, 0).width, 2);
    assert_eq!(grid.cell(2, 0).width, 0);
    assert_eq!(grid.row_text(0), "a中b ");
}

#[test]
fn wide_char_that_does_not_fit_is_not_written() {
    let mut grid = Grid::new(3, 1);
    assert_eq!(grid.put_str(0, 0, "ab中", Style::default(), 10), 2);
    assert_eq!(grid.row_text(0), "ab ");
}

#[test]
fn overwriting_half_of_wide_char_blanks_the_other_half() {
    let mut grid = Grid::new(4, 1);
    grid.put_str(0, 0, "中", Style::default(), 10);
    grid.put_str(1, 0, "x", Style::default(), 10);
    assert_eq!(grid.row_text(0), " x  ");
    grid.put_str(2, 0, "中", Style::default(), 10);
    grid.put_str(2, 0, "y", Style::default(), 10);
    assert_eq!(grid.row_text(0), " xy ");
    assert_eq!(grid.cell(3, 0).width, 1);
}

#[test]
fn grapheme_clusters_stay_in_one_cell() {
    let mut grid = Grid::new(4, 1);
    assert_eq!(grid.put_str(0, 0, "e\u{301}x", Style::default(), 10), 2);
    assert_eq!(grid.cell(0, 0).symbol.as_str(), "e\u{301}");
    let family = "👩\u{200d}👩\u{200d}👧";
    assert_eq!(grid.put_str(0, 0, family, Style::default(), 10), 2);
    assert_eq!(grid.cell(0, 0).symbol.as_str(), family);
}

#[test]
fn oversized_symbol_becomes_replacement_char() {
    let long = "👩\u{200d}👩\u{200d}👧\u{200d}👦";
    assert_eq!(Symbol::new(long).as_str(), "\u{fffd}");
    assert_eq!(Symbol::new("ab").as_str(), "ab");
}

#[test]
fn fill_and_clear_cover_only_the_rect() {
    let mut grid = Grid::new(4, 2);
    grid.put_str(0, 0, "abcd", Style::default(), 10);
    grid.fill(
        Rect {
            x: 1,
            y: 0,
            w: 2,
            h: 2,
        },
        red(),
    );
    assert_eq!(grid.row_text(0), "a  d");
    assert_eq!(grid.cell(2, 1).style, red());
    assert_eq!(grid.cell(3, 1).style, Style::default());
    grid.clear(Rect {
        x: 1,
        y: 0,
        w: 1,
        h: 1,
    });
    assert_eq!(*grid.cell(1, 0), Cell::default());
    assert_eq!(grid.cell(2, 0).style, red());
}

#[test]
fn border_draws_frame_with_title() {
    let mut grid = Grid::new(10, 3);
    grid.border(
        Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 3,
        },
        " ab ",
        red(),
    );
    assert_eq!(grid.row_text(0), "┌ ab ────┐");
    assert_eq!(grid.row_text(1), "│        │");
    assert_eq!(grid.row_text(2), "└────────┘");
    assert_eq!(grid.cell(0, 0).style, red());
}

#[test]
fn resize_resets_to_blank() {
    let mut grid = Grid::new(2, 1);
    grid.put_str(0, 0, "ab", red(), 10);
    grid.resize(3, 2);
    assert_eq!((grid.w, grid.h), (3, 2));
    assert_eq!(grid.row_text(0), "   ");
    assert_eq!(grid.row_text(1), "   ");
}
