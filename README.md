Prerequisites:
- EventStore Docker image (docker run --name esdb-node -it -p 2113:2113 -p 1113:1113 eventstore/eventstore:latest --insecure --run-projections=All --mem-db)
- MongoDB (docker run --name some-mongo -p 27017:27017 -d mongo)
- More brains than me (that's not a tall order)

TODO:
- docker compose
