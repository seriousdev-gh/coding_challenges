https://codingchallenges.fyi/challenges/challenge-url-shortener/

## Commands

 - `cargo run --bin db_setup` - Create database and run migrations
 - `cargo run` - Start server

 
 - `cargo run --bin db_drop` - Drop database

## Development

### Create new entities from migrations
 - `sea-orm-cli generate entity -o src/entities`