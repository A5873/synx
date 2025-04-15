# Example of an invalid Dockerfile with common errors
# This file should fail validation with Synx

# Missing FROM statement at the beginning

# Invalid instruction
WORKDIR /app

# Improper use of RUN (trying to run multiple commands without proper continuation)
RUN apt-get update
RUN apt-get install -y nodejs npm

# Now adding the missing FROM (out of order)
FROM ubuntu:latest

# Deprecated instruction
MAINTAINER John Doe <john@example.com>

# Bad practice: Running as root with no user specified

# Using ADD when COPY would be better
ADD https://example.com/app.tar.gz /app/

# Using latest tag
FROM node:latest

# Syntax error (missing closing quote)
ENV NODE_ENV="production

# Invalid chown format
COPY --chown=invalid:format app/ /app/

# Command with arguments should use JSON array format
CMD npm start

# Multiple CMD instructions (only the last one will be used)
CMD ["npm", "start"]

# Using sudo in a container (bad practice)
RUN sudo apt-get update

# Not cleaning up after apt-get
RUN apt-get install -y python

# Using apt-get upgrade (not recommended in containers)
RUN apt-get upgrade -y

# Missing --no-cache in apk add
RUN apk add nodejs

# Exposing a non-numeric port
EXPOSE http

# Using environment variables before defining them
RUN echo $UNDEFINED_VARIABLE

# Multiple ENTRYPOINT instructions
ENTRYPOINT ./app
ENTRYPOINT ["./app"]

# Invalid healthcheck
HEALTHCHECK incorrect

