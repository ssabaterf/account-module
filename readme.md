ROCKET_PORT=3000
ROCKET_ADDRESS=0.0.0.0
ROCKET_LOG=normal
ROCKET_ENV=production
DBURI=mongodb://localhost:27017
DBNAME=rocket
JWT_REFRESH_EXPIRES_IN=2592000
JWT_EXPIRES_IN=3600
JWT_REFRESH=refreshtokennoobextrasecure
JWT_SECRET=mysupersecret

cargo run

cargo build --release

docker build -t account-service:latest .

docker tag account-service:latest 123456789012.dkr.ecr.us-east-1.amazonaws.com/account-service:latest

docker push 123456789012.dkr.ecr.us-east-1.amazonaws.com/account-service:latest

docker run -d -p 3000:3000 \
-e ROCKET_PORT=3000 -e ROCKET_ADDRESS=0.0.0.0 \
-e DBURI=mongodb://localhost:27017 -e DBNAME=rocket\
-e JWT_REFRESH_EXPIRES_IN=2592000 -e JWT_EXPIRES_IN=3600 \
-e JWT_REFRESH=refreshtokennoobextrasecure -e JWT_SECRET=mysupersecret \
 account-service:latest accservice 