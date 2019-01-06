use snake_rust::game::Game;

fn main() {

    let mut game: Game = Game::new();

    game.setup();

    game.play();

    game.end();

}
