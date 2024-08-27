https://codingchallenges.fyi/challenges/challenge-url-shortener/

Result: http://seriousdevs.ru:5000/

## Commands

 - `docker compose stop` - Stop server and db

### On windows
 - `./docker/dev_setup.ps1` - Setup dev database
 - `./docker/dev_up.ps1` - Run server
 
 - `./docker/test_setup.ps1` - Setup test database
 - `./docker/test_run.ps1` - Run tests

### On linux
 - `./docker/dev_setup` - Setup dev database
 - `./docker/dev_up` - Run server
 
 - `./docker/test_setup` - Setup test database
 - `./docker/test_run` - Run tests

## Development

### Create new entities from migrations
 - `sea-orm-cli generate entity -o src/entities`

.