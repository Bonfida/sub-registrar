<h1 align="center">SNS Subdomain Registrar</h1>
<br />
<p align="center">
<img width="250" src="https://bafybeigmoph2jbhw4hjbqqgfj453tenw25g5je6ps35tftfe4tyil2k2re.ipfs.dweb.link/"/>
</p>
<p align="center">
<a href="https://twitter.com/bonfida">
<img src="https://img.shields.io/twitter/url?label=Bonfida&style=social&url=https%3A%2F%2Ftwitter.com%2Fbonfida">
</a>
</p>
<br />

The SNS Subdomain Registrar is a smart contract repository that provides functionalities for subdomain registration.

## Program ID

- Mainnet program ID `2KkyPzjaAYaz2ojQZ9P3xYakLd96B5UH6a2isLaZ4Cgs`

## Design

The SNS Subdomain Registrar allows .sol domain owners to establish a subdomain registrar, referred to as "registrar". Each registrar is a Program Derived Account (PDA) of the smart contract tasked with managing subdomain issuance. Upon the creation of a registrar, ownership of the .sol domain is transferred to the registrar's account to facilitate subdomain management. The creator of the registrar is required to define several parameters during its setup:

- **Price Schedule**: This outlines the cost for registering subdomains, which varies based on the length of the subdomain.
- **Admin Authority**: Identifies the individual or entity with permissions to modify the registrar's settings.
- **NFT Mint**: For scenarios where subdomain issuance is restricted to holders of specific NFT collections, the admin must specify a cap on the number of subdomains per NFT.
- **Revocation Rights**: Determines whether the admin has the authority to revoke previously issued subdomains.

To reclaim the .sol domain and dissolve the registrar, it is mandatory to delete all associated subdomains.

Subdomain issuance is monitored through `SubRecord` accounts, which track each subdomain. In cases where issuance is limited to NFT holders, `MintRecord` accounts are used to keep a count of subdomains issued per NFT, ensuring adherence to the specified limits.

## Reproducible build

A reproducible build script (`build.sh`) can be used to build the program using docker

## Security

For security disclosures or to report a bug, please visit [ImmuneFi](https://immunefi.com/bounty/bonfida/) for more information on our bug bounty program.
