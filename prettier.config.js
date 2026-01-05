export default {
  printWidth: 100,
  tabWidth: 4,
  useTabs: false,
  semi: true,
  singleQuote: false,
  quoteProps: "consistent",
  trailingComma: "es5",
  bracketSpacing: true,
  arrowParens: "always",
  endOfLine: "lf",
  proseWrap: "never",
  plugins: [],
  overrides: [
    {
      files: ["*.json", "*.json5"],
      options: {
        printWidth: 120,
        trailingComma: "none",
      },
    },
    {
      files: ["*.md", "*.markdown"],
      options: {
        proseWrap: "always",
        printWidth: 120,
      },
    },
  ],
};