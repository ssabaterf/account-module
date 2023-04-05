ROCKET_PORT=3000
ROCKET_ADDRESS=0.0.0.0
ROCKET_LOG=normal
ROCKET_ENV=production
DBURI=mongodb://localhost:27017
DBNAME=rocket


cargo run

cargo build --release

docker build -t account-service:latest .

docker tag account-service:latest 123456789012.dkr.ecr.us-east-1.amazonaws.com/account-service:latest

docker push 123456789012.dkr.ecr.us-east-1.amazonaws.com/account-service:latest

docker run -d -p 3000:3000 -e ROCKET_PORT=3000 -e ROCKET_ADDRESS=0.0.0.0 -e DBURI=mongodb://localhost:27017 -e DBNAME=rocket account-service:latest accservice