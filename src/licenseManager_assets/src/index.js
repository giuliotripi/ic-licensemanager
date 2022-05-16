import { licenseManager } from "../../declarations/licenseManager";

document.querySelector("form").addEventListener("submit", async (e) => {
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
document.querySelector("#refresh").addEventListener("click", async (e) => {
  const elencoLicenze = await licenseManager.list_products();
  let finalHtml = "";
  elencoLicenze.forEach(licenza => finalHtml += "<tr>" + licenza.id + " -> " + licenza.price + "€ </tr>\n");

  document.getElementById("elencoLicenze").innerHTML = finalHtml;
  return false;
});