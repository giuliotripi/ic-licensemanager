# License manager

This is an example project for Blockchain and cryptocurrencies exam @ University of Bologna.

Should be a Single Page application

## Authentication

In order to test the authentication with Internet Identity you either need a physical FIDO key, or
use Mozilla setting:
```
security.webauth.webauthn_enable_softtoken=true
security.webauth.webauthn_enable_usbtoken=false
```

[source](https://stackoverflow.com/questions/52445624/how-to-use-webauthn-without-key-fob)


## References

https://github.com/rocklabs-io/ic-py

https://github.com/rocklabs-io/ic-py/tree/main/examples Candid type - Python type

https://axios-http.com/docs/post_example

https://internetcomputer.org/docs/current/developer-docs/functionality/internet-identity/integrate-identity/ useful for deploy command of II on local

https://github.com/dfinity/internet-identity/blob/main/docs/internet-identity-spec.adoc#client-auth-protocol docs of II
https://github.com/dfinity/internet-identity/blob/main/docs/internet-identity-spec.adoc#deploying-on-testnets not so clear

https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app/agent/modules.html
https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app/auth-client/index.html
https://erxue-5aaaa-aaaab-qaagq-cai.ic0.app/identity/index.html
https://erxue-5aaaa-aaaab-qaagq-cai.ic0.app/principal/index.html



https://getbootstrap.com/docs/4.0/components/forms/#how-it-works
https://stackoverflow.com/questions/50658125/how-to-get-the-child-of-a-element-with-event-target
https://getbootstrap.com/docs/4.0/components/buttons/

## Run II in local

It is important that host is localhost, otherwise it displays a white page!!!!!!!

eg if you deploy the server on test.local it display a white page

if you connect to localhost everything works

```bash
# After checking out dfinity/internet-identity, run this in `./demos/using-dev-build`:
$ dfx start --background --clean
$ npm ci
$ dfx deploy --no-wallet --argument '(null)'
```

### Execution

If you have it in another computer and deploy it via ssh:

```bash
rsync -a IC/ test.local:project


ssh test.local -L 8000:localhost:8000

```

This way you can test it in your local PC using localhost as host.

## Summary

This example demonstrates a dapp consisting of two canister smart contracts:

* a simple backend canister, `licenseManager`, implementing the logic of the application in Rust, and
* a simple frontend asset canister, `licenseManager_assets` serving the assets of the dapp's web user interface.

This example is based on the default project created by running
`dfx new --type=rust hello` as described more fully
[here](https://smartcontracts.org/docs/rust-guide/rust-quickstart.html).


## Interface

Canister `licenseManager` is defined as a Rust library:

* [src/licenseManager/lib.rs](src/licenseManager/lib.rs)

with the Candid interface specified in [src/licenseManager/licenseManager.did](src/licenseManager/licenseManager.did)

The frontend displays a page with in HTML.

The relevant frontend code is:

* [src/licenseManager_assets/src/index.html](src/licenseManager_assets/src/index.html)
* [src/licenseManager_assets/src/index.jsx](src/licenseManager_assets/src/index.js)


## Requirements

The example requires an installation of:

* [DFINITY Canister SDK](https://sdk.dfinity.org).
* `node.js` (to build the web frontend).
* a suitable installaton of `rust` and `cmake` (see [here](https://smartcontracts.org/docs/rust-guide/rust-quickstart.html#before-you-begin)).

## Setup

Check, you have stopped any local canister execution environment (i.e. `replica`) or other network process that would create a port conflict on 8000.

## Running Locally

Using two terminal windows, do the following steps:

1. Open the first terminal window.

1. Start a local canister execution environment

   ```text
   dfx start
   ```

   This command produces a lot of distracting diagnostic output which is best ignored by continuing in a second terminal.

1. Open the second terminal window.

1. Ensure that the required `node` modules are available in your project directory, if needed, by running the following command:

   ```text
   npm install
   ```

1. Register, build and deploy the project.

   ```text
   dfx deploy
   ```

1. Call the hello canister's greet function:

   ```bash
   YOU=$(dfx identity get-principal)
   
   dfx canister call licenseManager mintDip721 \
    "(\"Seconda licenza\",principal\"$YOU\",vec{record{
        purpose=variant{Rendered};
        data=blob\"Seconda Licenza\";
        key_val_data=vec{
            record{
                \"contentType\";
                variant{TextContent=\"text/plain\"};
            };
            record{
                \"locationType\";
                variant{Nat8Content=4:nat8}
            };
            record{
                \"expire_date\";
                variant{TextContent=\"01-06-2022\"}
            };
        }
    }})"
   
   dfx canister call licenseManager CheckNfts '(0)'
   ```

1. Observe the result.


The previous steps use `dfx` to directly call the function on the `licenseManager` (backend) canister.

To access the web user interface of the dapp, that is served by canister `licenseManager_assets`, do the following:

1. Determine the URL of the `licenseManager_assets` asset canister.

   ```text
   echo "http://localhost:8000/?canisterId=$(dfx canister id licenseManager_assets)"
   ```

1. Navigate to the URL in your browser.

2. The browser should display HTML page with a single page application.

## Troubleshooting

If the web page doesn't display properly, or displays the wrong contents,
you may need to clear your browser cache.

Alternatively, open the URL in a fresh, in-private browser window to start with a clean cache.

## Links

For instructions on how to create this example from scratch as well as a more detailed walkthrough see:

- [Hello, World! Rust CDK Quick Start](https://smartcontracts.org/docs/rust-guide/rust-quickstart.html)

Other related links you might find useful are:

- [Rust Canister Development Guide](https://smartcontracts.org/docs/rust-guide/rust-intro.html)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://smartcontracts.org/docs/candid-guide/candid-intro.html)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app)
- [Troubleshoot issues](https://smartcontracts.org/docs/developers-guide/troubleshooting.html)

