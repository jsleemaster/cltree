use claude_explorer::vterm::VirtualTerminal;
use ratatui::prelude::*;

#[test]
fn test_basic_text_output() {
    let mut vt = VirtualTerminal::new(20, 5);
    vt.feed(b"Hello, World!");
    let grid = vt.grid();
    let text: String = grid[0].iter().take(13).map(|c| c.ch.as_str()).collect();
    assert_eq!(text, "Hello, World!");
}

#[test]
fn test_cursor_position_after_text() {
    let mut vt = VirtualTerminal::new(20, 5);
    vt.feed(b"ABC");
    assert_eq!(vt.cursor().x, 3);
    assert_eq!(vt.cursor().y, 0);
}

#[test]
fn test_cup_cursor_positioning() {
    let mut vt = VirtualTerminal::new(20, 10);
    // ESC[5;10H -> move to row 5, col 10 (1-indexed)
    vt.feed(b"\x1b[5;10H");
    assert_eq!(vt.cursor().y, 4);
    assert_eq!(vt.cursor().x, 9);

    vt.feed(b"X");
    assert_eq!(vt.grid()[4][9].ch, "X");
}

#[test]
fn test_cursor_movement_relative() {
    let mut vt = VirtualTerminal::new(20, 10);
    vt.feed(b"\x1b[5;10H"); // Move to (4, 9)

    // Move up 2
    vt.feed(b"\x1b[2A");
    assert_eq!(vt.cursor().y, 2);
    assert_eq!(vt.cursor().x, 9);

    // Move down 1
    vt.feed(b"\x1b[1B");
    assert_eq!(vt.cursor().y, 3);

    // Move right 3
    vt.feed(b"\x1b[3C");
    assert_eq!(vt.cursor().x, 12);

    // Move left 5
    vt.feed(b"\x1b[5D");
    assert_eq!(vt.cursor().x, 7);
}

#[test]
fn test_erase_display_below() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"AAAAAAAAAA\r\nBBBBBBBBBB\r\nCCCCCCCCCC");

    // Move to row 2, col 5, erase below
    vt.feed(b"\x1b[2;6H\x1b[0J");

    // Row 0 intact
    assert_eq!(vt.grid()[0][0].ch, "A");
    // Row 1, first 5 chars intact
    assert_eq!(vt.grid()[1][4].ch, "B");
    // Row 1, col 5+ erased
    assert_eq!(vt.grid()[1][5].ch, " ");
    // Row 2 erased
    assert_eq!(vt.grid()[2][0].ch, " ");
}

#[test]
fn test_erase_display_above() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"AAAAAAAAAA\r\nBBBBBBBBBB\r\nCCCCCCCCCC");

    // Move to row 2, col 3, erase above
    vt.feed(b"\x1b[2;4H\x1b[1J");

    // Row 0 erased
    assert_eq!(vt.grid()[0][0].ch, " ");
    // Row 1 up to col 3 erased
    assert_eq!(vt.grid()[1][3].ch, " ");
    // Row 1, col 4+ intact
    assert_eq!(vt.grid()[1][4].ch, "B");
    // Row 2 intact
    assert_eq!(vt.grid()[2][0].ch, "C");
}

#[test]
fn test_erase_display_all() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"AAAAAAAAAA\r\nBBBBBBBBBB\r\nCCCCCCCCCC");
    vt.feed(b"\x1b[2J");

    for row in vt.grid() {
        for cell in row {
            assert_eq!(cell.ch, " ");
        }
    }
}

#[test]
fn test_erase_line() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"ABCDEFGHIJ");

    // Move to col 5, erase to end
    vt.feed(b"\x1b[1;6H\x1b[0K");
    assert_eq!(vt.grid()[0][4].ch, "E");
    assert_eq!(vt.grid()[0][5].ch, " ");
    assert_eq!(vt.grid()[0][9].ch, " ");

    // Erase from start to cursor
    vt.feed(b"\x1b[1;4H\x1b[1K");
    assert_eq!(vt.grid()[0][0].ch, " ");
    assert_eq!(vt.grid()[0][3].ch, " ");
    assert_eq!(vt.grid()[0][4].ch, "E");
}

#[test]
fn test_sgr_foreground_colors() {
    let mut vt = VirtualTerminal::new(20, 5);

    // Red text
    vt.feed(b"\x1b[31mR");
    assert_eq!(vt.grid()[0][0].style.fg, Some(Color::Red));

    // Green text
    vt.feed(b"\x1b[32mG");
    assert_eq!(vt.grid()[0][1].style.fg, Some(Color::Green));

    // Reset
    vt.feed(b"\x1b[0mN");
    assert_eq!(vt.grid()[0][2].style, Style::default());
}

#[test]
fn test_sgr_bold_italic() {
    let mut vt = VirtualTerminal::new(20, 5);
    vt.feed(b"\x1b[1;3mBI");
    let style = vt.grid()[0][0].style;
    assert!(style.add_modifier.contains(Modifier::BOLD));
    assert!(style.add_modifier.contains(Modifier::ITALIC));
}

#[test]
fn test_sgr_256_color() {
    let mut vt = VirtualTerminal::new(20, 5);
    // 256-color foreground: ESC[38;5;196m
    vt.feed(b"\x1b[38;5;196mX");
    assert_eq!(vt.grid()[0][0].style.fg, Some(Color::Indexed(196)));
}

#[test]
fn test_sgr_rgb_color() {
    let mut vt = VirtualTerminal::new(20, 5);
    // RGB foreground: ESC[38;2;255;128;0m
    vt.feed(b"\x1b[38;2;255;128;0mX");
    assert_eq!(vt.grid()[0][0].style.fg, Some(Color::Rgb(255, 128, 0)));
}

#[test]
fn test_scroll_on_overflow() {
    let mut vt = VirtualTerminal::new(5, 3);
    vt.feed(b"A\r\nB\r\nC\r\nD\r\nE");

    // After 5 lines in 3-row terminal, 2 lines should be in scrollback
    assert_eq!(vt.scrollback().len(), 2);
    assert_eq!(vt.scrollback()[0][0].ch, "A");
    assert_eq!(vt.scrollback()[1][0].ch, "B");

    // Grid should have last 3 lines
    assert_eq!(vt.grid()[0][0].ch, "C");
    assert_eq!(vt.grid()[1][0].ch, "D");
    assert_eq!(vt.grid()[2][0].ch, "E");
}

#[test]
fn test_line_wrap() {
    let mut vt = VirtualTerminal::new(5, 3);
    vt.feed(b"ABCDEFGH");
    // Wraps at col 5
    assert_eq!(vt.grid()[0][4].ch, "E");
    assert_eq!(vt.grid()[1][0].ch, "F");
    assert_eq!(vt.grid()[1][2].ch, "H");
}

#[test]
fn test_alternate_screen_buffer() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"Main content");

    // Enter alternate screen
    vt.feed(b"\x1b[?1049h");
    // Main screen content should be saved, grid should be blank
    assert_eq!(vt.grid()[0][0].ch, " ");

    vt.feed(b"Alt content");
    assert_eq!(vt.grid()[0][0].ch, "A");

    // Leave alternate screen
    vt.feed(b"\x1b[?1049l");
    assert_eq!(vt.grid()[0][0].ch, "M");
    assert_eq!(vt.grid()[0][1].ch, "a");
}

#[test]
fn test_cursor_visibility() {
    let mut vt = VirtualTerminal::new(10, 5);
    assert!(vt.cursor().visible);

    // Hide cursor
    vt.feed(b"\x1b[?25l");
    assert!(!vt.cursor().visible);

    // Show cursor
    vt.feed(b"\x1b[?25h");
    assert!(vt.cursor().visible);
}

#[test]
fn test_resize() {
    let mut vt = VirtualTerminal::new(10, 5);
    vt.feed(b"Hello World");
    vt.resize(5, 3);

    assert_eq!(vt.cols(), 5);
    assert_eq!(vt.rows(), 3);
    // Existing content preserved within new bounds
    assert_eq!(vt.grid()[0][0].ch, "H");
    assert_eq!(vt.grid()[0][4].ch, "o");
}

#[test]
fn test_carriage_return_overwrite() {
    let mut vt = VirtualTerminal::new(10, 5);
    vt.feed(b"Hello\rWorld");
    assert_eq!(vt.grid()[0][0].ch, "W");
    assert_eq!(vt.grid()[0][1].ch, "o");
    assert_eq!(vt.grid()[0][2].ch, "r");
}

#[test]
fn test_tab_stops() {
    let mut vt = VirtualTerminal::new(20, 5);
    vt.feed(b"A\tB");
    assert_eq!(vt.grid()[0][0].ch, "A");
    // Tab should advance to next 8-column boundary
    assert_eq!(vt.grid()[0][8].ch, "B");
}

#[test]
fn test_backspace() {
    let mut vt = VirtualTerminal::new(10, 5);
    vt.feed(b"AB\x08C");
    assert_eq!(vt.grid()[0][0].ch, "A");
    assert_eq!(vt.grid()[0][1].ch, "C"); // B overwritten by C
}

#[test]
fn test_delete_characters() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"ABCDEF");
    // Move to col 2, delete 2 chars
    vt.feed(b"\x1b[1;3H\x1b[2P");
    assert_eq!(vt.grid()[0][0].ch, "A");
    assert_eq!(vt.grid()[0][1].ch, "B");
    assert_eq!(vt.grid()[0][2].ch, "E");
    assert_eq!(vt.grid()[0][3].ch, "F");
}

#[test]
fn test_insert_lines() {
    let mut vt = VirtualTerminal::new(5, 3);
    vt.feed(b"A\r\nB\r\nC");
    // Move to row 2, insert 1 line
    vt.feed(b"\x1b[2;1H\x1b[1L");
    assert_eq!(vt.grid()[0][0].ch, "A");
    assert_eq!(vt.grid()[1][0].ch, " "); // Inserted blank
    assert_eq!(vt.grid()[2][0].ch, "B"); // Pushed down
}

#[test]
fn test_delete_lines() {
    let mut vt = VirtualTerminal::new(5, 3);
    vt.feed(b"A\r\nB\r\nC");
    // Move to row 2, delete 1 line
    vt.feed(b"\x1b[2;1H\x1b[1M");
    assert_eq!(vt.grid()[0][0].ch, "A");
    assert_eq!(vt.grid()[1][0].ch, "C"); // Row 3 moved up
    assert_eq!(vt.grid()[2][0].ch, " "); // New blank row at bottom
}

#[test]
fn test_save_restore_cursor() {
    let mut vt = VirtualTerminal::new(10, 5);
    vt.feed(b"\x1b[3;5H"); // Move to (2,4)

    // Save cursor via ESC 7
    vt.feed(b"\x1b7");
    vt.feed(b"\x1b[1;1H"); // Move to (0,0)
    assert_eq!(vt.cursor().y, 0);
    assert_eq!(vt.cursor().x, 0);

    // Restore cursor via ESC 8
    vt.feed(b"\x1b8");
    assert_eq!(vt.cursor().y, 2);
    assert_eq!(vt.cursor().x, 4);
}

#[test]
fn test_scroll_offset() {
    let mut vt = VirtualTerminal::new(5, 3);
    // Generate scrollback
    for i in 0..10 {
        vt.feed(format!("{}\r\n", i).as_bytes());
    }

    assert!(vt.scrollback().len() > 0);

    vt.set_scroll_offset(3);
    assert_eq!(vt.scroll_offset(), 3);

    // Cannot exceed scrollback length
    vt.set_scroll_offset(10000);
    assert_eq!(vt.scroll_offset(), vt.scrollback().len());
}

#[test]
fn test_cha_cursor_horizontal_absolute() {
    let mut vt = VirtualTerminal::new(20, 5);
    vt.feed(b"ABCDEFGHIJ");
    // CHA: move to column 5 (1-indexed)
    vt.feed(b"\x1b[5G");
    assert_eq!(vt.cursor().x, 4);
}

#[test]
fn test_vpa_vertical_position_absolute() {
    let mut vt = VirtualTerminal::new(20, 10);
    // VPA: move to row 5 (1-indexed)
    vt.feed(b"\x1b[5d");
    assert_eq!(vt.cursor().y, 4);
}

#[test]
fn test_erase_characters() {
    let mut vt = VirtualTerminal::new(10, 3);
    vt.feed(b"ABCDEFGHIJ");
    // Move to col 3, erase 3 characters
    vt.feed(b"\x1b[1;4H\x1b[3X");
    assert_eq!(vt.grid()[0][2].ch, "C");
    assert_eq!(vt.grid()[0][3].ch, " ");
    assert_eq!(vt.grid()[0][4].ch, " ");
    assert_eq!(vt.grid()[0][5].ch, " ");
    assert_eq!(vt.grid()[0][6].ch, "G");
}

#[test]
fn test_complex_sgr_sequence() {
    let mut vt = VirtualTerminal::new(20, 5);
    // Bold + Red FG + Blue BG in single sequence
    vt.feed(b"\x1b[1;31;44mX");
    let style = vt.grid()[0][0].style;
    assert!(style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(style.fg, Some(Color::Red));
    assert_eq!(style.bg, Some(Color::Blue));
}

#[test]
fn test_combining_character() {
    let mut vt = VirtualTerminal::new(20, 5);
    // "é" = e (U+0065) + combining acute accent (U+0301)
    vt.feed("é".as_bytes());
    assert_eq!(vt.grid()[0][0].ch, "é");
    assert_eq!(vt.cursor().x, 1); // Only 1 cell consumed
}

#[test]
fn test_variation_selector() {
    let mut vt = VirtualTerminal::new(20, 5);
    // U+269B (atom symbol) + U+FE0F (variation selector 16)
    vt.feed("⚛\u{FE0F}".as_bytes());
    assert!(vt.grid()[0][0].ch.contains('⚛'));
    assert!(vt.grid()[0][0].ch.contains('\u{FE0F}'));
}
