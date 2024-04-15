#!/bin/bash
# Check if the Docker image exists
set -e -v
docker build -t sub_register .

solana program dump namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX target/deploy/spl_name_service.so
solana program dump jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR target/deploy/sns_registrar.so
solana program dump metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s target/deploy/mpl_token_metadata.so




if [[ ${1} == "build-only" ]]; then
    echo "Only building..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir --mount type=bind,source=$SSH_AUTH_SOCK,target=/ssh-agent --env SSH_AUTH_SOCK=/ssh-agent --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse sub_register:latest /bin/bash -c "cargo build-sbf"
elif [[ ${1} == "test" ]]; then
    echo "Running tests..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir --mount type=bind,source=$SSH_AUTH_SOCK,target=/ssh-agent --env SSH_AUTH_SOCK=/ssh-agent --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse sub_register:latest /bin/bash -c "cargo test-sbf"
else
    echo "Running tests + building..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir --mount type=bind,source=$SSH_AUTH_SOCK,target=/ssh-agent --env SSH_AUTH_SOCK=/ssh-agent --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse sub_register:latest /bin/bash -c "cargo test-sbf && cargo build-sbf"
fi