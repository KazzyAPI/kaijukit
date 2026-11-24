module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "scope-enum": [1, "always", ["core", "adf", "server", "web", "ui", "editor", "conformance", "deploy", "docs", "ci", "deps"]],
    "subject-case": [2, "never", ["start-case", "pascal-case", "upper-case"]],
    "header-max-length": [2, "always", 72],
  },
};
