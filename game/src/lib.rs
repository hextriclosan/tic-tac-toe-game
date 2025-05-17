use wasm_bindgen::prelude::*;
use web_sys::{window, Document};
use std::cell::RefCell;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    set_event_handlers()?;
    Ok(())
}

thread_local! {
    static GAME: RefCell<GameState> = RefCell::new(GameState::new());
}

struct GameState {
    board: [[char; 3]; 3],
    turn: char,
    over: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            board: [[' '; 3]; 3],
            turn: 'X',
            over: false,
        }
    }

    fn reset(&mut self) {
        self.board = [[' '; 3]; 3];
        self.turn = 'X';
        self.over = false;
        clear_result();
    }
}

fn document() -> Document {
    window().unwrap().document().unwrap()
}

fn set_result(message: &str) {
    if let Some(elem) = document().get_element_by_id("result") {
        elem.set_inner_html(message);
    }
}

#[wasm_bindgen]
pub fn click_cell(row: usize, col: usize) -> Result<(), JsValue> {
    GAME.with(|game| {
        let mut game = game.borrow_mut();

        // Prevent clicking after game is over
        if game.over || game.board[row][col] != ' ' {
            return Ok(());
        }

        let doc = document();
        let id = format!("cell-{}-{}", row, col);
        let cell = doc.get_element_by_id(&id).unwrap();
        cell.set_inner_html(&game.turn.to_string());

        game.board[row][col] = game.turn;

        let board = game.board;

        // Prepare next turn (if any)
        game.turn = if game.turn == 'X' { 'O' } else { 'X' };

        // Schedule win check
        let closure = Closure::<dyn FnMut()>::new(move || {
            if let Some((winner, win_cells)) = check_winner(&board) {
                set_result(&format!("{} wins!", winner));

                let doc = document();
                for (r, c) in win_cells {
                    let cell_id = format!("cell-{}-{}", r, c);
                    if let Some(cell) = doc.get_element_by_id(&cell_id) {
                        let _ = cell.class_list().add_1("win-cell");
                    }
                }

                GAME.with(|g| g.borrow_mut().over = true);
            }
            else if is_tie(&board) {
                set_result("It's a tie!");
                GAME.with(|g| g.borrow_mut().over = true);
            }
        });

        window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                0,
            )?;
        closure.forget();

        Ok(())
    })
}



fn check_winner(board: &[[char; 3]; 3]) -> Option<(char, Vec<(usize, usize)>)> {
    // Check rows
    for i in 0..3 {
        if board[i][0] != ' ' && board[i][0] == board[i][1] && board[i][1] == board[i][2] {
            return Some((board[i][0], vec![(i, 0), (i, 1), (i, 2)]));
        }
    }
    // Check columns
    for j in 0..3 {
        if board[0][j] != ' ' && board[0][j] == board[1][j] && board[1][j] == board[2][j] {
            return Some((board[0][j], vec![(0, j), (1, j), (2, j)]));
        }
    }
    // Check diagonals
    if board[0][0] != ' ' && board[0][0] == board[1][1] && board[1][1] == board[2][2] {
        return Some((board[0][0], vec![(0, 0), (1, 1), (2, 2)]));
    }
    if board[0][2] != ' ' && board[0][2] == board[1][1] && board[1][1] == board[2][0] {
        return Some((board[0][2], vec![(0, 2), (1, 1), (2, 0)]));
    }
    None
}

fn clear_result() {
    if let Some(elem) = document().get_element_by_id("result") {
        elem.set_inner_html("");
    }
}


fn is_tie(board: &[[char; 3]; 3]) -> bool {
    board.iter().all(|row| row.iter().all(|&c| c != ' '))
}

fn set_event_handlers() -> Result<(), JsValue> {
    let doc = document();

    let reset_btn = doc.get_element_by_id("reset-btn").unwrap();
    let closure = Closure::wrap(Box::new(move || {
        let _ = reset_game();
    }) as Box<dyn FnMut()>);

    reset_btn
        .dyn_ref::<web_sys::HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));

    closure.forget();

    for row in 0..3 {
        for col in 0..3 {
            let id = format!("cell-{}-{}", row, col);
            let cb = Closure::<dyn FnMut()>::new(move || {
                let _ = click_cell(row, col);
            });
            doc.get_element_by_id(&id)
                .unwrap()
                .dyn_ref::<web_sys::HtmlElement>()
                .unwrap()
                .set_onclick(Some(cb.as_ref().unchecked_ref()));
            cb.forget(); // Leak to keep it alive
        }
    }
    Ok(())
}

#[wasm_bindgen]
pub fn reset_game() -> Result<(), JsValue> {
    GAME.with(|game| {
        let mut game = game.borrow_mut();
        game.reset();

        let doc = document();

        // Clear cells and remove win highlight
        for row in 0..3 {
            for col in 0..3 {
                let id = format!("cell-{}-{}", row, col);
                if let Some(cell) = doc.get_element_by_id(&id) {
                    let html_element = cell.dyn_into::<web_sys::HtmlElement>()?;
                    html_element.set_inner_html("");
                    html_element.class_list().remove_1("win-cell")?;
                }
            }
        }

        // Clear result message
        if let Some(result) = doc.get_element_by_id("result") {
            result.set_inner_html("");
        }

        Ok(())
    })
}
