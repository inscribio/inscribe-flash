module.exports = {
  // Prevent wild errors on Windows when git automatically changed line endings
  endOfLine: process.platform.startsWith('win') ? "auto" : "lf",
}
