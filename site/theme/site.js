// The pages of the site: screenshots opened large in place, and the theme
// switcher. Without JavaScript the screenshots and themes are plain links to
// the images.
(function () {
  "use strict";
  var shots = document.querySelectorAll(".shot a");
  if (shots.length) {
    var lightbox = document.createElement("dialog");
    lightbox.className = "lightbox";
    var image = document.createElement("img");
    image.alt = "";
    lightbox.appendChild(image);
    document.body.appendChild(lightbox);
    shots.forEach(function (link) {
      link.addEventListener("click", function (event) {
        event.preventDefault();
        image.src = link.href;
        image.alt = link.querySelector("img").alt;
        lightbox.showModal();
      });
    });
    lightbox.addEventListener("click", function () { lightbox.close(); });
  }

  var view = document.querySelector(".theme-view img[data-alt]");
  document.querySelectorAll(".themes a").forEach(function (link) {
    link.addEventListener("click", function (event) {
      event.preventDefault();
      var current = document.querySelector(".themes [aria-current]");
      if (current) current.removeAttribute("aria-current");
      link.setAttribute("aria-current", "true");
      view.src = link.href;
      view.alt = view.dataset.alt.replace("{}", link.textContent);
    });
  });
})();
