//! The cell size in pixels from the window size.

use mu_tui::screen::cell_pixels;
use ratatui::backend::WindowSize;
use ratatui::layout::Size;

fn window(cells: (u16, u16), pixels: (u16, u16)) -> WindowSize {
    WindowSize {
        columns_rows: Size::new(cells.0, cells.1),
        pixels: Size::new(pixels.0, pixels.1),
    }
}

#[test]
fn cell_is_the_window_over_its_cells() {
    assert_eq!(
        cell_pixels(&window((80, 24), (800, 480))),
        Some(Size::new(10, 20))
    );
}

#[test]
fn no_pixel_size_gives_no_cell() {
    let sizes = [
        window((80, 24), (0, 0)),
        window((80, 24), (800, 0)),
        window((0, 0), (800, 480)),
    ];
    assert_eq!(sizes.map(|size| cell_pixels(&size)), [None; 3]);
}
