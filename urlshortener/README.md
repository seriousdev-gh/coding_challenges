https://codingchallenges.fyi/challenges/challenge-url-shortener/

## Commands

### On windows
#### Start server
 - `$env:APP_ENV="dev"; cargo run --bin db_setup` - Create database and run migrations
 - `$env:APP_ENV="dev"; cargo run` - Start server

#### Run tests
 - `$env:APP_ENV="test"; cargo run --bin db_setup` - Create database and run migrations
 - `$env:APP_ENV="test"; cargo test` - Run tests

 - `cargo run --bin db_drop` - Drop database

## Development

### Create new entities from migrations
 - `sea-orm-cli generate entity -o src/entities`