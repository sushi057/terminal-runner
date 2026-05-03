mod game;
mod player;
mod terminal;
mod world;

fn main() {
    let mut term = terminal::Terminal::init();
    let mut game = game::Game::new(term.width(), term.height());
    game.run(&mut term);
}
