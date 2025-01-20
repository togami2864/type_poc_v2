use server::Server;

fn main() {
    let server = Server::new();
    let path = std::env::current_dir().unwrap();
    let path = path.join("src/tests/index.ts");
    server.init(vec![path]).unwrap();

    server.debug();
}
