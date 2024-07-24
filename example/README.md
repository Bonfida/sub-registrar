# SNS Subdomain Registrar Demo
This is a minimal react/nextjs implementation of server side subdomain registration through SNS sub-registrar.

## Prerequisites
- Node.js version 18 or above
- SNS domain
- Full access to Solana RPC

## Installation

1. Clone the repository from [here](https://github.com/Bonfida/sub-registrar).

2. Navigate to the example folder under the cloned repository.

3. Install the dependencies by running the following command:

        npm install

4. Register a domain on [SNS](https://www.sns.id/) and create a sub-registrar using the registered domain.

5. Copy the .env.example file and rename it to .env. Configure the .env file as follows:
    - PRIVATE_KEY: Private key of sub-registrar admin as a bs58 encoded string
    - NEXT_PUBLIC_RPC: Solana RPC endpoint
    - NEXT_PUBLIC_DOMAIN_NAME: Domain name of the sub-registrar

## Usage

To start the server in development mode, run the following command:

    npm run dev

The server will start running at http://localhost:3000.

## Dependencies

- @bonfida/emojis: ^1.0.4
- @bonfida/spl-name-service: 2.5.4
- @bonfida/sub-register: ^0.0.1-alpha.8
- @solana/wallet-adapter-base: ^0.9.23
- @solana/wallet-adapter-react: ^0.15.35
- @solana/wallet-adapter-react-ui: ^0.9.35
- @solana/web3.js: ^1.95.1
- bs58: ^6.0.0
- next: 14.2.5
- react: ^18
- react-dom: ^18

## Bugs and Issues

If you encounter any bugs or issues, please report them [here](https://github.com/Bonfida/sub-registrar/issues).

## License

This project is licensed under the ISC License. See the LICENSE file for more information.