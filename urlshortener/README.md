https://codingchallenges.fyi/challenges/challenge-url-shortener/

## Commands

### On windows
 - ./docker/dev_setup.ps1 - Setup dev database
 - ./docker/dev_up.ps1 - Run server
 
 - ./docker/test_setup.ps1 - Setup test database
 - ./docker/test_run.ps1 - Run tests

## Development

### Create new entities from migrations
 - `sea-orm-cli generate entity -o src/entities`