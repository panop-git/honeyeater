// Injects a version + last-updated line into the footer of every page.
//
// This file is regenerated at build time by .cloudflare/build-docs.sh, which
// substitutes the real values from Cargo.toml and git. The placeholder values
// below keep a local `mdbook serve` working without the build wrapper.
(function () {
  var VERSION = "__HONEYEATER_VERSION__";
  var UPDATED = "__HONEYEATER_UPDATED__";

  // Leave the placeholders alone during a bare local build so nobody mistakes
  // a dev preview for a released version.
  var version = VERSION.indexOf("__HONEYEATER") === 0 ? "dev" : VERSION;
  var updated = UPDATED.indexOf("__HONEYEATER") === 0 ? "" : UPDATED;

  document.addEventListener("DOMContentLoaded", function () {
    var content = document.querySelector(".content");
    if (!content) return;

    var line = document.createElement("p");
    line.className = "honeyeater-version";
    line.textContent = updated
      ? "honeyeater " + version + " · docs updated " + updated
      : "honeyeater " + version;

    content.appendChild(line);
  });
})();
