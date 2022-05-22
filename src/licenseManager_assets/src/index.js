import { licenseManager } from "../../declarations/licenseManager";
import cryptoRandomString from 'crypto-random-string';

const axios = require("axios");
const baseUrl = "http://192.168.1.130:8080";
import { loadScript } from "@paypal/paypal-js";

let referenceId, amount, customId;

let elencoLicenze;

document.querySelector("form#nameForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  document.getElementById("greeting").innerText = "";
  const loader = document.getElementById("loader");

  const button = e.target.querySelector("button");

  const name = document.getElementById("name").value.toString();

  loader.style.visibility = "visible";
  button.setAttribute("disabled", true);
  document.getElementById("name").setAttribute("disabled", true);

  // Interact with foo actor, calling the greet method
  const greeting = await licenseManager.greet(name);

  loader.style.visibility = "hidden";
  button.removeAttribute("disabled");
  document.getElementById("name").removeAttribute("disabled");
  document.getElementById("greeting").innerText = greeting;

  return false;
});

//aggiunge una licenza all'elenco degli elementi acquistabili
document.querySelector("form#updateForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  document.getElementById("greeting").innerText = "";
  const loader = document.getElementById("loader");

  const button = e.target.querySelector("button");

  const licenseName = document.getElementById("licenseId").value.toString();
  const cost = document.getElementById("licensePrice").value.toString();

  loader.style.visibility = "visible";
  button.setAttribute("disabled", true);

  const result = await licenseManager.update({id: licenseName, price: parseFloat(cost)});

  loader.style.visibility = "hidden";
  button.removeAttribute("disabled");
  document.getElementById("greeting").innerText = result;

  return false;
});
document.querySelector("#refresh").addEventListener("click", async (e) => {
  elencoLicenze = await licenseManager.list_products();
  let finalHtmlSelect = "";

  generateTable();

  elencoLicenze.forEach(licenza =>
      finalHtmlSelect += `<option value='${licenza.id}' data-description='${licenza.id}' data-referenceId='${licenza.id}' data-price='${licenza.price}'>${licenza.id} -> ${licenza.price}€ </option>\n`);

  document.getElementById("elencoLicenze").style.visibility = elencoLicenze.length > 0 ? "visible" : "hidden";

  document.getElementById("elencoLicenzeBuy").innerHTML = finalHtmlSelect;
  return false;
});

document.querySelector("#searchLicense").addEventListener("change", async (e) => {
  generateTable();
});
document.querySelector("#searchLicense").addEventListener("keyup", async (e) => {
  generateTable();
});

function generateTable() {
  let finalHtml = "";

  let filtro = document.getElementById("searchLicense").value;

  elencoLicenze.filter(licenza => licenza.id.includes(filtro)).forEach(licenza => finalHtml += "<tr><td>" + licenza.id + "</td><td>" + licenza.price + "€ </td></tr>\n");

  document.querySelector("#elencoLicenze > tbody").innerHTML = finalHtml;
}

document.querySelector("#buyForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  document.getElementById("paypal-button-container").style.visibility = "visible";
  return false;
});

function callExternalServer(id, email, payerId) {
  axios.get(baseUrl + "/check", {params: {id, referenceId, amount, customId, email, payerId}}).then(function (response) {
    console.log(response.data);
    if(response.data.ok) {
      console.log("Transazione conclusa con successo");
    } else {
      alert("Transazione fallita: " + response.data.message);
    }
  }).catch(function (response) {

  });
}

import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";

const webapp_id = process.env.WHOAMI_CANISTER_ID;

// The interface of the whoami canister
const webapp_idl = ({ IDL }) => {
  return IDL.Service({ whoami: IDL.Func([], [IDL.Principal], ["query"]) });
};
export const init = ({ IDL }) => {
  return [];
};

// Autofills the <input> for the II Url to point to the correct canister.
document.body.onload = () => {
  let iiUrl;
  if (process.env.DFX_NETWORK === "local") {
    iiUrl = `http://localhost:8000/?canisterId=${process.env.II_CANISTER_ID}`;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.II_CANISTER_ID}.ic0.app`;
  } else {
    iiUrl = `https://${process.env.II_CANISTER_ID}.dfinity.network`;
  }
  document.getElementById("iiUrl").value = iiUrl;
};

document.getElementById("loginBtn").addEventListener("click", async () => {
  // When the user clicks, we start the login process.
  // First we have to create and AuthClient.
  const authClient = await AuthClient.create();

  // Find out which URL should be used for login.
  const iiUrl = document.getElementById("iiUrl").value;

  // Call authClient.login(...) to login with Internet Identity. This will open a new tab
  // with the login prompt. The code has to wait for the login process to complete.
  // We can either use the callback functions directly or wrap in a promise.
  await new Promise((resolve, reject) => {
    authClient.login({
      identityProvider: iiUrl,
      onSuccess: resolve,
      onError: reject,
    });
  });

  // At this point we're authenticated, and we can get the identity from the auth client:
  const identity = authClient.getIdentity();
  // Using the identity obtained from the auth client, we can create an agent to interact with the IC.
  const agent = new HttpAgent({ identity });
  // Using the interface description of our webapp, we create an actor that we use to call the service methods.
  const webapp = Actor.createActor(webapp_idl, {
    agent,
    canisterId: webapp_id,
  });
  // Call whoami which returns the principal (user id) of the current user.
  const principal = await webapp.whoami();
  // show the principal on the page
  document.getElementById("loginStatus").innerText = principal.toText();
});


loadScript({ "client-id": "AVU3VIXs5KxLh3u6zXANqSwG9t53d3agoElb-z3ploa6ooLTmDst2nPIq0xHSvesGNH3Yy4mrQbXaCoZ", "components": "buttons", "currency": "EUR" })
    .then((paypal) => {
      paypal.Buttons({
        style: {
          layout: 'vertical',
          color:  'blue',
          shape:  'rect',
          label:  'paypal'
        },
        createOrder: function(data, actions) {
          // Set up the transaction
          //https://developer.paypal.com/docs/api/orders/v2/#definition-purchase_unit_request
          console.log("Setting up the transaction...");
          const selectLicenze = document.getElementById("elencoLicenzeBuy");
          const selectedElement = selectLicenze.options[selectLicenze.selectedIndex];
          if(selectedElement === undefined) {
            alert("Nessun elemento selezionato");
            return;
          }
          const description = selectedElement.getAttribute("data-description");
          referenceId = selectedElement.getAttribute("data-referenceId");
          amount = parseFloat(selectedElement.dataset.price);
          amount = amount.toFixed(2);
          customId = cryptoRandomString({length: 100, type: "alphanumeric"});
          return actions.order.create({
            purchase_units: [{
              amount: {
                value: amount,
                currency_code: "EUR"
              },
              description: description,
              custom_id: customId, //Appears in transaction and settlement reports but is not visible to the payer.
              reference_id: referenceId,
              // invoice_id: "invoice" + Math.floor(Math.random() * 100000000000000000000000000)
            }]
          });
        },
        onApprove: function(data, actions) {
          // This function captures the funds from the transaction.
          return actions.order.capture().then(function(details) {
            // This function shows a transaction success message to your buyer.
            console.log('Transaction completed by ' + details.payer.name.given_name);
            callExternalServer(details.id, details.payer.email_address, details.payer.payer_id);
            console.log(details);
            document.getElementById("paypal-button-container").style.visibility = "hidden";
          });
        }
      }).render('#paypal-button-container');
    })
    .catch((err) => {
      console.error("failed to load the PayPal JS SDK script", err);
    });