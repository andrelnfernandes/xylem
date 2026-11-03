// The site in another language, at the same page. The books are / (English)
// and /<lang>/; mdBook gives each page the way to its book's root.
(function () {
  "use strict";
  var languages = [
    ["en", "English"], ["pt", "Português"], ["es", "Español"],
    ["fr", "Français"], ["de", "Deutsch"], ["zh", "中文"]
  ];
  var bar = document.querySelector(".right-buttons");
  if (!bar || typeof path_to_root === "undefined") return;
  var current = document.documentElement.lang || "en";
  var book = new URL(path_to_root || "./", window.location.href);
  var site = current === "en" ? book : new URL("../", book);
  var page = window.location.href.slice(book.href.length).split("#")[0];

  var select = document.createElement("select");
  select.className = "xylem-language";
  select.setAttribute("aria-label", "Language");
  languages.forEach(function (l) {
    var option = document.createElement("option");
    option.value = l[0];
    option.textContent = l[1];
    option.lang = l[0];
    option.selected = l[0] === current;
    select.appendChild(option);
  });
  select.addEventListener("change", function () {
    var lang = select.value;
    window.location.href = new URL((lang === "en" ? "" : lang + "/") + page, site).href;
  });
  bar.insertBefore(select, bar.firstChild);
})();
