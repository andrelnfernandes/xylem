// The manual in another language, at the same topic, and the way back to the
// home page. The books are /manual/ (English) and /manual/<lang>/.
(function () {
  "use strict";
  var languages = [
    ["en", "English"], ["pt", "Português"], ["es", "Español"],
    ["fr", "Français"], ["de", "Deutsch"], ["zh", "中文"]
  ];
  var path = window.location.pathname;
  var at = path.lastIndexOf("/manual/");
  if (at < 0) return;
  var base = path.slice(0, at + "/manual/".length);
  var rest = path.slice(base.length);
  var current = "en";
  languages.forEach(function (l) {
    if (l[0] !== "en" && rest.indexOf(l[0] + "/") === 0) {
      current = l[0];
      rest = rest.slice(l[0].length + 1);
    }
  });
  var bar = document.querySelector(".right-buttons");
  if (!bar) return;

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
    window.location.href = base + (lang === "en" ? "" : lang + "/") + rest + window.location.hash;
  });
  bar.insertBefore(select, bar.firstChild);

  var home = document.createElement("a");
  home.className = "xylem-home";
  home.href = base.slice(0, -"manual/".length) + (current === "pt" ? "pt-br/" : "");
  home.textContent = "Xylem";
  home.title = "Home page";
  bar.insertBefore(home, bar.firstChild);
})();
