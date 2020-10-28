FROM ubuntu:20.04

RUN apt-get update && apt install -y \
  git \
  vim \
  gcc \
  make \
  ranger \
  silversearcher-ag \
  htop \
  curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
COPY . /api

RUN USER=root cargo new --bin rust-api

EXPOSE 4000

# Set environment variables.
ENV HOME /api

# Define working directory.
WORKDIR /api

RUN cargo build

# Define default command.
CMD ["cargo run"]

