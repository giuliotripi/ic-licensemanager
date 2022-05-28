import { licenseManager } from "../../declarations/licenseManager";
import cryptoRandomString from 'crypto-random-string';

const axios = require("axios");
const baseUrl = "http://192.168.1.130:8080";
import { loadScript } from "@paypal/paypal-js";
import { Actor, HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";

let referenceId, amount, customId;

let elencoLicenze;

let lastLicenseListRefresh = 0;

//navbar
//only one onload function is supported at a time
document.body.onload = () => {
  let hash = window.location.hash;
  console.log(hash);
  displaySection(hash);
  checkIIcanisterId();
};

//TODO: prende solo il primo elemento, querySel restituisce una lista sulla quale operare
document.querySelector(".nav-link").addEventListener("click", async (e) => {
  // e.preventDefault();
  // console.log(e);
  // console.log(this);
  // displaySection(this.getAttribute("href"));
  // return false;
});


//pagination
window.addEventListener('hashchange', async function () {
  await displaySection(window.location.hash);
}, false);

async function displaySection(sectionName) {
  if(sectionName.length > 0 && sectionName[0] === "#")
    sectionName = sectionName.substring(1);
  $("main.container").css("display", "none");
  // document.querySelector("main.container").style.display = "none"; //solo sul primo
  switch (sectionName) {
    case "":
    case "home":
      document.getElementById("home").style.display = "inherit";
      break;
    case "licenses":
      document.getElementById("licenses").style.display = "inherit";
      break;
    case "buy":
      document.getElementById("buy").style.display = "inherit";
      await refreshLicenseList();
      break;
    case "add":
      document.getElementById("add").style.display = "inherit";
      break;
    default:
      document.getElementById("home").style.display = "inherit";
      break;
  }
}

//aggiunge una licenza all'elenco degli elementi acquistabili
document.querySelector("form#addLicenseForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  document.getElementById("result").innerText = "";

  const button = e.target.querySelector("button");
  // console.log(button);
  // console.log(e.target);
  // console.log(e.target.querySelector(".loadingIcon"));
  // console.log(button.querySelector(".loadingIcon"));
  // const $loader = $("#addLicenseForm .loadingIcon");
  const loader = e.target.querySelector(".loadingIcon");

  const licenseId = document.getElementById("inputLicenseId").value;
  const licenseName = document.getElementById("inputLicenseName").value;
  const licensePrice = document.getElementById("inputLicensePrice").value;
  const licenseDescription = document.getElementById("inputLicenseDescription").value;
  const licenseDuration = document.getElementById("inputLicenseDuration").value;
  const licenseTransferFees = document.getElementById("inputLicenseTransferFees").value;
  const licenseTransferrable = document.getElementById("inputLicenseCanBeTransferred").checked;
  const perpetual = parseInt(licenseDuration) === 0;

  // $loader.css("display", "initial");
  loader.style.display = "initial";
  button.setAttribute("disabled", true);

  const result = await licenseManager.update({id: licenseId, price: parseFloat(licensePrice), name: licenseName, description: licenseDescription, duration: parseInt(licenseDuration), perpetual: perpetual, transfer_commission: parseFloat(licenseTransferFees), transferable: licenseTransferrable});

  // $loader.css("display", "none");
  loader.style.display = "none";
  button.removeAttribute("disabled");
  document.getElementById("result").innerText = result;

  return false;
});

import { faker } from '@faker-js/faker';

document.getElementById("addLicenseCompileDefaults").addEventListener("click", async (e) => {
  document.getElementById("inputLicenseId").value = cryptoRandomString({length: 5, type: "alphanumeric"});
  document.getElementById("inputLicenseName").value = faker.name.firstName();
  document.getElementById("inputLicensePrice").value = parseInt(cryptoRandomString({length: 3, type: "numeric"}))/10;
  document.getElementById("inputLicenseDescription").value = faker.lorem.words(10);
  document.getElementById("inputLicenseDuration").value = 0;
  document.getElementById("inputLicenseTransferFees").value = 0;
  document.getElementById("inputLicenseCanBeTransferred").checked = true;
});


document.querySelector("#refresh").addEventListener("click", async (e) => {

  await refreshLicenseList(e);

  return false;
});

async function refreshLicenseList(e) {
  let loader;
  let button;
  if(e !== undefined) {
    loader = e.target.querySelector(".loadingIcon");
    button = e.target;
  } else {
    button = document.querySelector("#refresh");
    loader = document.querySelector("#refresh .loadingIcon");
  }

  button.setAttribute("disabled", true);
  loader.style.display = "initial";

  elencoLicenze = await licenseManager.list_products();

  lastLicenseListRefresh = Date.now();

  loader.style.display = "none";
  button.removeAttribute("disabled");

  generateTable();

  document.getElementById("elencoLicenze").style.visibility = elencoLicenze.length > 0 ? "visible" : "hidden";
}

document.querySelector("#searchLicense").addEventListener("change", async (e) => {
  generateTable();
});
document.querySelector("#searchLicense").addEventListener("keyup", async (e) => {
  generateTable();
});

function generateTable() {
  let finalHtml = "";

  let filtro = document.getElementById("searchLicense").value;

  elencoLicenze.filter(licenza => licenza.id.includes(filtro)).slice(0, 30).forEach(
      licenza => finalHtml += "<tr><td>" + licenza.id + "</td><td>" + licenza.name + "</td><td>" + licenza.price + "€</td><td>" + licenza.duration + "</td>" +
          "<td class='buyElement' data-license-id='" + licenza.id + "'>Buy</td></tr>\n"
  );

  document.querySelector("#elencoLicenze > tbody").innerHTML = finalHtml;
}

$("#elencoLicenze").on("click", ".buyElement",function (e) {
  $("#searchAndListLicenses").css("display", "none");
  $("#checkout").css("display", "block");
  let finalHtml = "";
  let licenseId = e.target.getAttribute("data-license-id");
  let license = elencoLicenze.find(l => l.id === licenseId);

  let duration = (license.duration === 0) ? "unlimited" : license.duration + " days";

  finalHtml += `<tr><td>ID</td><td>${license.id}</td></tr>`;
  finalHtml += `<tr><td>Name</td><td>${license.name}</td></tr>`;
  finalHtml += `<tr><td>Price</td><td>${license.price}</td></tr>`;
  finalHtml += `<tr><td>Description</td><td>${license.description}</td></tr>`;
  finalHtml += `<tr><td>Transferable</td><td>${license.transferable}</td></tr>`;
  finalHtml += `<tr><td>Transfer commission</td><td>${license.transfer_commission}</td></tr>`;
  finalHtml += `<tr><td>Duration</td><td>${duration}</td></tr>`;
  $("#licenseDetails").html(finalHtml);
});

document.getElementById("cancelCheckout").addEventListener("click", async (e) => {
  $("#searchAndListLicenses").css("display", "");
  $("#checkout").css("display", "none");
});

function callExternalServer(id, email, payerId) {
  //https://axios-http.com/docs/post_example
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

const webapp_id = process.env.WHOAMI_CANISTER_ID;

// The interface of the whoami canister
const webapp_idl = ({ IDL }) => {
  return IDL.Service({ whoami: IDL.Func([], [IDL.Principal], ["query"]) });
};
export const init = ({ IDL }) => {
  return [];
};

// Autofills the <input> for the II Url to point to the correct canister.
function checkIIcanisterId() {
  let iiUrl;
  if (process.env.DFX_NETWORK === "local") {
    iiUrl = `http://localhost:8000/?canisterId=${process.env.II_CANISTER_ID}`;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.II_CANISTER_ID}.ic0.app`;
  } else {
    iiUrl = `https://${process.env.II_CANISTER_ID}.dfinity.network`;
  }
  // document.getElementById("iiUrl").value = iiUrl;
  console.log(process.env, process.env.DFX_NETWORK, process.env.II_CANISTER_ID)
}

document.getElementById("loginBtn").addEventListener("click", async () => {

  // When the user clicks, we start the login process.
  // First we have to create and AuthClient.
  const authClient = await AuthClient.create();

  // Find out which URL should be used for login.
  // const iiUrl = document.getElementById("iiUrl").value;

  // Call authClient.login(...) to login with Internet Identity. This will open a new tab
  // with the login prompt. The code has to wait for the login process to complete.
  // We can either use the callback functions directly or wrap in a promise.
  await new Promise((resolve, reject) => {
    authClient.login({
      identityProvider: "http://localhost:8000/?canisterId=qoctq-giaaa-aaaaa-aaaea-cai",
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
